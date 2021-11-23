#[cfg(target_os = "windows")]
use super::paths::get_regkey_path;
use super::paths::{get_manifest_path, Browser};
use super::targets::chrome::ChromeHost;
use super::targets::firefox::FirefoxHost;
use crate::config::NAME;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub fn install_manifest(browser: &Browser, path: Option<PathBuf>) -> Result<PathBuf, String> {
    // Create native messaging path if it doesn't already exist
    let manifest_path = match path {
        Some(p) => Ok(p),
        None => get_manifest_path(&browser),
    }?;

    fs::create_dir_all(&manifest_path)
        .map_err(|_| "Failed to create native messaging directory.")?;

    // Determine path of self/executable
    let exe_path = std::env::current_exe()
        .map_err(|_| "Failed to determine location of executable.")?
        .into_os_string()
        .into_string()
        .map_err(|_| "Failed to serialise location of executable.")?;

    // Create JSON file
    let filename = NAME.to_owned() + ".json";
    let full_write_path = manifest_path.join(filename);
    let mut file =
        fs::File::create(&full_write_path).map_err(|_| "Failed to create manifest file.")?;

    // Write manifest to created file
    let manifest = match browser {
        Browser::Firefox | Browser::LibreWolf => serde_json::to_string(&FirefoxHost::new(exe_path)),
        _ => serde_json::to_string(&ChromeHost::new(exe_path)),
    }
    .map_err(|_| "Failed to serialise manifest.")?;

    file.write_all(manifest.as_bytes())
        .map_err(|_| "Failed to write to manifest file.")?;

    // Register regkey
    #[cfg(target_os = "windows")]
    register_regkey(&browser, &full_write_path)?;

    Ok(full_write_path)
}

#[cfg(target_os = "windows")]
const REGKEY: &str = NAME;

#[cfg(target_os = "windows")]
fn register_regkey(browser: &Browser, json_path: &PathBuf) -> Result<(), &'static str> {
    let path_prefix = get_regkey_path(&browser).ok_or("Failed to get regkey path.")?;
    let path = PathBuf::from(path_prefix).join(REGKEY);

    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(&path)
        .map_err(|_| "Failed to create registry entry.")?;

    key.set_value("", &json_path.to_string_lossy().into_owned())
        .map_err(|_| "Failed to set registry entry.")?;

    Ok(())
}
