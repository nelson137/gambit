use std::{
    fs::{create_dir_all, File},
    io::{Cursor, Read, Seek, Write},
    os::unix::prelude::OpenOptionsExt,
    path::Path,
    process::ExitCode,
};

use anyhow::{bail, Context, Result};
use reqwest;
use zip::ZipArchive;

/// The working directory for acquiring the Stockfish source code.
const STOCKFISH_DIR: &str = "target/stockfish";

/// The base name of the zip file containing the Stockfish source code.
const STOCKFISH_ZIP_NAME: &str = "stockfish_14_linux_x64.zip";

/// The base URL of the server from which the Stockfish release will be
/// downloaded. This value concatenated with `STOCKFISH_ZIP_NAME` is the full
/// URL from which to download.
const STOCKFISH_ZIP_URL: &str = "https://stockfishchess.org/files/";

/// The path to the Stockfish binary in the release ZIP archive.
const STOCKFISH_ZIP_BIN_PATH: &str = "stockfish_14_linux_x64/stockfish_14_x64";

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/stockfish");

    // return ExitCode::FAILURE;
    match main2() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}

fn main2() -> Result<()> {
    ensure_stockfish_dir()?;

    ensure_stockfish()?;

    Ok(())
}

fn ensure_stockfish_dir() -> Result<()> {
    create_dir_all(STOCKFISH_DIR).with_context(|| {
        format!("Failed to ensure that the Stockfish working directory exists: {STOCKFISH_DIR}")
    })
}

fn ensure_stockfish() -> Result<()> {
    let zip_p = Path::new(STOCKFISH_DIR).join(STOCKFISH_ZIP_NAME);

    if zip_p.exists() {
        if !zip_p.is_file() {
            bail!(
                "Unable to download Stockfish binary, path exists and is not a file: {}",
                zip_p.display()
            );
        }

        let archive_f = File::open(&zip_p).with_context(|| {
            format!("Failed to open Stockfish ZIP archive: {}", zip_p.display())
        })?;

        return ensure_stockfish_binary(&zip_p, archive_f);
    }

    let url = format!("{STOCKFISH_ZIP_URL}{STOCKFISH_ZIP_NAME}");
    let mut response = reqwest::blocking::get(url.clone())
        .with_context(|| format!("Failed to get the Stockfish ZIP archive: {url}"))?
        .error_for_status()
        .with_context(|| "Server returned error for Stockfish ZIP archive")?;

    let mut buf = Vec::new();
    response
        .copy_to(&mut buf)
        .with_context(|| "Failed to write downloaded Stockfish ZIP archive to buffer")?;

    File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&zip_p)
        .with_context(|| {
            format!("Failed to open output file for downloaded Stockfish ZIP archive: {url}")
        })?
        .write_all(&buf)
        .with_context(|| {
            format!("Failed to write downloaded Stockfish ZIP archive to file: {}", zip_p.display())
        })?;

    ensure_stockfish_binary(&zip_p, Cursor::new(buf))
}

fn ensure_stockfish_binary(zip_p: &Path, zip_reader: impl Read + Seek) -> Result<()> {
    let zip_bin_p = Path::new(STOCKFISH_ZIP_BIN_PATH);
    let bin_name = zip_bin_p
        .file_name()
        .expect("Failed to get basename of Stockfish binary file path in the ZIP archive -- this should never happen, the path is const");

    let bin_p = Path::new(STOCKFISH_DIR).join(bin_name);
    if bin_p.exists() {
        if bin_p.is_file() {
            return Ok(());
        } else {
            bail!(
                "Unable to extract Stockfish binary from ZIP archive, path exists and is not a file: {}",
                bin_p.display()
            );
        }
    }

    let mut archive = ZipArchive::new(zip_reader)
        .with_context(|| format!("Failed to load ZIP archive: {}", zip_p.display()))?;

    let mut bin = archive.by_name(STOCKFISH_ZIP_BIN_PATH).with_context(|| {
        format!("Failed to find Stockfish binary in ZIP archive: {STOCKFISH_ZIP_BIN_PATH}")
    })?;
    if !bin.is_file() {
        bail!("The path of the Stockfish binary in the ZIP archive is not a file: {STOCKFISH_ZIP_BIN_PATH}");
    }

    let mut buf = Vec::new();
    bin.read_to_end(&mut buf).with_context(|| "Failed to copy the Stockfish binary to a buffer")?;

    File::options()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o775)
        .open(&bin_p)
        .with_context(|| {
            format!("Failed to open the file for the Stockfish binary: {}", bin_p.display())
        })?
        .write_all(&buf)
        .with_context(|| {
            format!("Failed to write the Stockfish binary to file: {}", bin_p.display())
        })?;

    Ok(())
}
