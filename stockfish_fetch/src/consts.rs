/// The name of the log file. This value joined with `WORKING_DIR` is the full
/// path.
pub const LOG_FILE_PATH: &str = "target/stockfish.log";

/// The working directory for this script.
pub const WORKING_DIR: &str = "target/stockfish";

/// The base URL of the server from which the Stockfish release will be
/// downloaded. This value joined with `STOCKFISH_ZIP_NAME` is the full URL.
pub const STOCKFISH_ZIP_URL: &str =
    "https://github.com/official-stockfish/Stockfish/archive/refs/tags/";

/// The name of the zip file containing the Stockfish source code repository.
/// This value joined with `WORKING_DIR` is the full path.
pub const STOCKFISH_ZIP_NAME: &str = "sf_15.zip";

/// The name of the Stockfish source code repository directory that will be
/// extracted from the ZIP archive. This value joined with `WORKING_DIR` is the
/// full path.
pub const STOCKFISH_REPO_DIR_NAME: &str = "Stockfish-sf_15";

/// The architecture argument for `make` when compiling Stockfish.
#[cfg(all(target_vendor = "apple", target_os = "macos", target_arch = "aarch64"))]
pub const STOCKFISH_ARCH: &str = "ARCH=apple-silicon";
#[cfg(not(target_arch = "aarch64"))]
pub const STOCKFISH_ARCH: &str = "ARCH=x86-64-modern";

/// The name of the Stockfish executable.
pub const STOCKFISH_BIN_NAME: &str = "stockfish";
