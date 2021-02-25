use platforms::target::{OS, TARGET_OS};
use std::env::{current_dir, var};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};
use std::path::PathBuf;

fn var_path(env_var: &str) -> Option<PathBuf> {
    var(env_var).map(PathBuf::from).ok()
}

/// Determine path to database from environment variables.
// Nota bene that this must exactly match the logic of Buku's internal
// `get_default_dbdir` function.
pub fn get_db_path() -> Result<PathBuf, IoError> {
    let dir = match TARGET_OS {
        OS::Windows => var_path("APPDATA"),
        _ => var_path("XDG_DATA_HOME")
            .or_else(|| var_path("HOME").map(|path| path.join(".local/share")))
            .or_else(|| current_dir().ok()),
    };

    dir.map(|data_path| data_path.join("buku/bookmarks.db"))
        .filter(|full_path| full_path.is_file())
        .ok_or_else(|| IoError::new(IoErrorKind::NotFound, "Failed to find Buku database."))
}
