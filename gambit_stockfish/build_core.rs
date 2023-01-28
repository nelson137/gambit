use std::{
    fs::{remove_dir_all, File},
    io::{Cursor, Read, Seek, Write},
    path::{Path, PathBuf},
    thread::available_parallelism,
};

use anyhow::{anyhow, Context, Result};
use tracing::{error, info};
use zip::ZipArchive;

use crate::{build_consts::*, build_utils::BuildCommand};

pub struct StockfishBuilder {
    zip_p: PathBuf,
    repo_dir_p: PathBuf,
    bin_p: PathBuf,
}

impl StockfishBuilder {
    pub fn new() -> Self {
        let zip_p = Path::new(WORKING_DIR).join(STOCKFISH_ZIP_NAME);
        let repo_dir_p = Path::new(WORKING_DIR).join(STOCKFISH_REPO_DIR_NAME);
        let mut bin_p = repo_dir_p.join("src");
        bin_p.push(STOCKFISH_BIN_NAME);
        Self { zip_p, repo_dir_p, bin_p }
    }

    pub fn run(&self) -> Result<()> {
        if self.bin_p.exists() {
            info!(path = %self.bin_p.display(), "Executable exists, nothing to do");
        } else {
            info!(path = %self.bin_p.display(), "Executable doesn't exist");

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
        info!(path = %self.zip_p.display(), "Source code archive exists, reading from filesystem");
        File::open(&self.zip_p).with_context(|| {
            format!("Failed to open source code archive: {}", self.zip_p.display())
        })
    }

    fn download_zip(&self) -> Result<impl Read + Seek> {
        let url = format!("{STOCKFISH_ZIP_URL}{STOCKFISH_ZIP_NAME}");
        info!(%url, "Source code archive doesn't exist, downloading now");

        let mut response = reqwest::blocking::get(url.clone())
            .with_context(|| format!("Failed to get source code archive: {url}"))?
            .error_for_status()
            .context("Server returned error")?;

        let mut buf = Vec::new();
        response
            .copy_to(&mut buf)
            .with_context(|| "Failed to copy source code archive to buffer")?;

        info!(path = %self.zip_p.display(), "Writing source code archive to filesystem");
        File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.zip_p)
            .with_context(|| format!("Failed to open output file for source code archive: {url}"))?
            .write_all(&buf)
            .with_context(|| {
                format!("Failed to write source code archive to file: {}", self.zip_p.display())
            })?;

        Ok(Cursor::new(buf))
    }

    fn extract_zip(&self, reader: impl Read + Seek) -> Result<()> {
        info!("Extracting archive");

        let mut archive = ZipArchive::new(reader).with_context(|| {
            format!("Failed to load soure code archive: {}", self.zip_p.display())
        })?;

        archive
            .extract(WORKING_DIR)
            .with_context(|| {
                format!("Failed to extract source code archive into directory: {WORKING_DIR}")
            })
            .map_err(|err| {
                if !self.repo_dir_p.exists() {
                    return err;
                }

                match remove_dir_all(&self.repo_dir_p) {
                    Ok(_) => info!(
                        path = %self.repo_dir_p.display(),
                        "Removed partially-extracted source code repository",
                    ),
                    Err(err) => error!(
                        path = %self.repo_dir_p.display(),
                        "Failed to remove partially-extracted source code repository ({err:#})",
                    ),
                }

                err
            })?;

        Ok(())
    }

    fn build_src(&self) -> Result<()> {
        info!(path = %self.repo_dir_p.display(), "Compiling executable");

        let make_dir_p = self.repo_dir_p.join("src");
        let make_dir_s = make_dir_p
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert path to string: {}", make_dir_p.display()))?;

        // Compile
        {
            let mut cmd = BuildCommand::new("make");
            cmd.args(["-C", make_dir_s, "net", "build", STOCKFISH_ARCH]);
            if let Ok(parallelism) = available_parallelism() {
                let parallelism = (usize::from(parallelism) / 2).max(1);
                cmd.arg("-j");
                cmd.arg(parallelism.to_string());
            }
            cmd.run()?;
        }

        // Strip binary
        {
            info!(path = %self.bin_p.display(), "Stripping executable");
            BuildCommand::new("make").args(["-C", make_dir_s, "strip"]).run()?;
        }

        Ok(())
    }
}
