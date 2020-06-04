use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;

/// Determine path to database from environment variables.
pub fn get_db_path() -> Result<PathBuf, IoError> {
    let db_filename = "bookmarks.db";

    let dir = match dirs::data_dir() {
        Some(data_dir) => Ok(data_dir.join("buku")),
        None => Err("Failed to locate data directory."),
    };

    let dir_and_file_existing = dir
        .ok()
        .map(|d| d.join(db_filename))
        .filter(|path| path.is_file());

    dir_and_file_existing
        .ok_or_else(|| IoError::new(IoErrorKind::NotFound, "Failed to find Buku database."))
}
