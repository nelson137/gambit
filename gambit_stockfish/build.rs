use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    process::ExitCode,
};

use anyhow::{Context, Result};
use build_core::StockfishBuilder;
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
