use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    process::ExitCode,
};

use anyhow::{Context, Result};
use build_core::StockfishBuilder;
use build_utils::LogWriter;
use tracing::error;
use tracing_subscriber::fmt::{time::LocalTime, SubscriberBuilder};

mod build_consts;
mod build_core;
mod build_utils;

use crate::build_consts::*;

fn main() -> ExitCode {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=target/stockfish");
    println!();

    // Open and prepare log file
    let log_file_p = Path::new(LOG_FILE_PATH);
    let mut log_file = match File::options().create(true).append(true).open(log_file_p) {
        Ok(f) => f,
        Err(err) => {
            eprintln!("Failed to open log file: {LOG_FILE_PATH}");
            eprintln!("\n    {err:#}");
            return ExitCode::FAILURE;
        }
    };
    writeln!(
        log_file,
        "\n--------------------------------[ BEGIN BUILD ]--------------------------------\n"
    )
    .unwrap();

    // Setup log subscriber
    SubscriberBuilder::default()
        .with_writer(LogWriter::new(log_file))
        .with_ansi(false)
        .with_timer(LocalTime::rfc_3339())
        .with_target(false)
        .init();

    match main2() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{:?}", err);
            ExitCode::FAILURE
        }
    }
}

fn main2() -> Result<()> {
    // Ensure that the script working directory in target/ exists
    create_dir_all(WORKING_DIR).with_context(|| {
        format!("Failed to ensure that the Stockfish working directory exists: {WORKING_DIR}")
    })?;

    // Core logic
    StockfishBuilder::new().run()
}
