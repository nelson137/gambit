use std::{
    fmt,
    fs::{create_dir_all, remove_dir_all, File},
    io::{Cursor, Read, Seek, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode},
    thread::available_parallelism,
};

use anyhow::{anyhow, bail, Context, Result};
use shell_escape::escape;
use tracing::{error, info};
use tracing_subscriber::fmt::{time::LocalTime, SubscriberBuilder};
use zip::ZipArchive;

/// The name of the log file. This value joined with `WORKING_DIR` is the full
/// path.
const LOG_FILE_PATH: &str = "target/stockfish.log";

/// The working directory for this script.
const WORKING_DIR: &str = "target/stockfish";

/// The base URL of the server from which the Stockfish release will be
/// downloaded. This value joined with `STOCKFISH_ZIP_NAME` is the full URL.
const STOCKFISH_ZIP_URL: &str =
    "https://github.com/official-stockfish/Stockfish/archive/refs/tags/";

/// The name of the zip file containing the Stockfish source code repository.
/// This value joined with `WORKING_DIR` is the full path.
const STOCKFISH_ZIP_NAME: &str = "sf_15.zip";

/// The name of the Stockfish source code repository directory that will be
/// extracted from the ZIP archive. This value joined with `WORKING_DIR` is the
/// full path.
const STOCKFISH_REPO_DIR_NAME: &str = "Stockfish-sf_15";

/// The architecture argument for `make` when compiling Stockfish.
#[cfg(all(target_vendor = "apple", target_os = "macos", target_arch = "aarch64"))]
const STOCKFISH_ARCH: &str = "ARCH=apple-silicon";
#[cfg(not(target_arch = "aarch64"))]
const STOCKFISH_ARCH: &str = "ARCH=x86-x64";

/// The name of the Stockfish executable.
const STOCKFISH_BIN_NAME: &str = "stockfish";

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/stockfish");
    println!();

    match main2() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{:?}", err);
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}

fn main2() -> Result<()> {
    // Ensure that the stockfish working directory in target/ exists
    create_dir_all(WORKING_DIR).with_context(|| {
        format!("Failed to ensure that the Stockfish working directory exists: {WORKING_DIR}")
    })?;

    // Open and prepare log file
    let mut log_file = File::options().create(true).append(true).open(Path::new(LOG_FILE_PATH))?;
    writeln!(
        log_file,
        "\n--------------------------------[ BEGIN BUILD ]--------------------------------\n"
    )?;

    // Setup log subscriber
    let timer = LocalTime::rfc_3339();
    SubscriberBuilder::default().with_ansi(false).with_timer(timer).with_writer(log_file).init();

    // Run build logic
    StockfishBuilder::new().run()
}

struct StockfishBuilder {
    zip_p: PathBuf,
    repo_dir_p: PathBuf,
    bin_p: PathBuf,
}

impl StockfishBuilder {
    fn new() -> Self {
        let zip_p = Path::new(WORKING_DIR).join(STOCKFISH_ZIP_NAME);
        let repo_dir_p = Path::new(WORKING_DIR).join(STOCKFISH_REPO_DIR_NAME);
        let mut bin_p = repo_dir_p.join("src");
        bin_p.push(STOCKFISH_BIN_NAME);
        Self { zip_p, repo_dir_p, bin_p }
    }

    fn run(&self) -> Result<()> {
        if self.bin_p.exists() {
            info!(path = %self.bin_p.display(), "executable exists, nothing to do");
        } else {
            // Ensure that the repository directory exists (i.e. is extracted from the zip archive)
            if !self.repo_dir_p.exists() {
                if self.zip_p.exists() {
                    self.extract_zip(self.read_zip()?)?;
                } else {
                    self.extract_zip(self.download_zip()?)?;
                }
            }

            self.build_src()?;
        }

        Ok(())
    }

    fn read_zip(&self) -> Result<impl Read + Seek> {
        info!(path = %self.zip_p.display(), "zip archive exists, reading into memory");
        File::open(&self.zip_p).with_context(|| {
            format!("Failed to open Stockfish ZIP archive: {}", self.zip_p.display())
        })
    }

    fn download_zip(&self) -> Result<impl Read + Seek> {
        let url = format!("{STOCKFISH_ZIP_URL}{STOCKFISH_ZIP_NAME}");
        info!(url, "zip archive doesn't exist, downloading now");

        let mut response = reqwest::blocking::get(url.clone())
            .with_context(|| format!("Failed to get the Stockfish source code ZIP archive: {url}"))?
            .error_for_status()
            .context("Server returned error for Stockfish source code ZIP archive")?;

        let mut buf = Vec::new();
        response.copy_to(&mut buf).with_context(|| {
            "Failed to write downloaded Stockfish source code ZIP archive to buffer"
        })?;

        info!(path = %self.zip_p.display(), "writing zip archive to filesystem");
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.zip_p)
            .with_context(|| {
                format!("Failed to open output file for downloaded Stockfish source code ZIP archive: {url}")
            })?
            .write_all(&buf)
            .with_context(|| {
                format!(
                    "Failed to write downloaded Stockfish source code ZIP archive to file: {}",
                    self.zip_p.display()
                )
            })?;

        Ok(Cursor::new(buf))
    }

    fn extract_zip(&self, reader: impl Read + Seek) -> Result<()> {
        info!("extracting zip archive");

        let mut archive = ZipArchive::new(reader)
            .with_context(|| format!("Failed to load ZIP archive: {}", self.zip_p.display()))?;

        archive
            .extract(WORKING_DIR)
            .with_context(|| {
                format!("Failed to extract Stockfish source code ZIP archive into directory: {WORKING_DIR}")
            })
            .map_err(|err| {
                if !self.repo_dir_p.exists() {
                    return err;
                }

                match remove_dir_all(&self.repo_dir_p) {
                    Ok(_) => eprint!(
                        "CLEANUP: Removed partially-extracted Stockfish source code directory: {}\n\n",
                        self.repo_dir_p.display()
                    ),
                    Err(rm_err) => eprint!(
                        "CLEANUP: Failed to remove partially-extracted Stockfish source code directory: {}\n    ({rm_err:#})\n\n",
                        self.repo_dir_p.display()
                    )
                }

                err
            })?;

        Ok(())
    }

    fn build_src(&self) -> Result<()> {
        info!(path = %self.repo_dir_p.display(), "executable doesn't exist, compiling now");

        let make_dir_p = self.repo_dir_p.join("src");
        let make_dir_s = make_dir_p
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert path to string: {}", make_dir_p.display()))?;

        let cmd_err = |cmd: &Command| format!("Failed to run command:\n\n    {}", cmd.display());

        // Compile
        {
            let mut cmd = Command::new("make");
            cmd.args(["-C", make_dir_s, "net", "build", STOCKFISH_ARCH]);
            if let Ok(parallelism) = available_parallelism() {
                cmd.arg("-j");
                cmd.arg(parallelism.to_string());
            }
            if !cmd.status().with_context(|| cmd_err(&cmd))?.success() {
                bail!(cmd_err(&cmd));
            }
        }

        // Strip binary
        {
            let mut cmd = Command::new("make");
            cmd.args(["-C", make_dir_s, "strip"]);
            info!(path = %self.bin_p.display(), "strip executable");
            if !cmd.status().with_context(|| cmd_err(&cmd))?.success() {
                bail!(cmd_err(&cmd));
            }
        }

        Ok(())
    }
}

trait CommandExts {
    fn display(&self) -> CommandDisplay<'_>;
}

impl CommandExts for Command {
    fn display(&self) -> CommandDisplay {
        CommandDisplay(self)
    }
}

struct CommandDisplay<'cmd>(&'cmd Command);

impl<'cmd> fmt::Display for CommandDisplay<'cmd> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.get_program().to_string_lossy())?;

        for arg in self.0.get_args() {
            write!(f, " {}", escape(arg.to_string_lossy()))?;
        }

        Ok(())
    }
}
