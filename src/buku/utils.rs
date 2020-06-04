use std::env::var;
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;

/// Determine path to database from environment variables.
pub fn get_db_path() -> Result<PathBuf, IoError> {
    let db_filename = "bookmarks.db";

    let dir = match var("XDG_DATA_HOME") {
        Ok(xdg_home) => Ok(PathBuf::from(xdg_home + "/buku/")),
        _ => match var("HOME") {
            Ok(home) => Ok(PathBuf::from(home + "/.local/share/buku/")),
            Err(err) => Err(err),
        },
    };

    let dir_and_file_existing = dir
        .ok()
        .map(|d| d.join(db_filename))
        .filter(|path| path.is_file());

    dir_and_file_existing
        .ok_or_else(|| IoError::new(IoErrorKind::NotFound, "Failed to find Buku database."))
}
