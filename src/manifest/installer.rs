#[cfg(target_os = "windows")]
use super::paths::get_regkey_path;
use super::paths::{get_manifest_path, Browser};
use super::targets::chrome::ChromeHost;
use super::targets::firefox::FirefoxHost;
use crate::config::NAME;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// This trait represents all side effects in the installation of a manifest.
/// Its purpose is both documentative and to ease testing.
pub trait InstallerEffects<A> {
    fn get_exe_path() -> Result<String, A>;
    fn get_manifest_path(browser: &Browser) -> Result<PathBuf, A>;
    fn create_manifest_dir(path: &Path) -> Result<(), A>;
    fn create_manifest_file(path: &Path) -> Result<fs::File, A>;
    fn generate_manifest(browser: &Browser, exe_path: &str) -> Result<String, A>;
    fn write_manifest_file(file: fs::File, data: &[u8]) -> Result<(), A>;
    #[cfg(target_os = "windows")]
    fn register_regkey(browser: &Browser, json_path: &Path) -> Result<(), A>;
}

/// This struct has the real, runtime implementation of `InstallerEffects`.
pub struct RuntimeInstaller;

#[cfg(target_os = "windows")]
const REGKEY: &str = NAME;

impl InstallerEffects<String> for RuntimeInstaller {
    fn get_exe_path() -> Result<String, String> {
        std::env::current_exe()
            .map_err(|_| "Failed to determine location of executable.")?
            .into_os_string()
            .into_string()
            .map_err(|_| "Failed to serialise location of executable.".into())
    }

    fn get_manifest_path(browser: &Browser) -> Result<PathBuf, String> {
        get_manifest_path(&browser)
    }

    fn create_manifest_dir(path: &Path) -> Result<(), String> {
        fs::create_dir_all(&path).map_err(|_| "Failed to create native messaging directory.".into())
    }

    fn create_manifest_file(path: &Path) -> Result<fs::File, String> {
        fs::File::create(&path).map_err(|_| "Failed to create manifest file.".into())
    }

    fn generate_manifest(browser: &Browser, exe_path: &str) -> Result<String, String> {
        match browser {
            Browser::Firefox => serde_json::to_string(&FirefoxHost::new(exe_path)),
            _ => serde_json::to_string(&ChromeHost::new(exe_path)),
        }
        .map_err(|_| "Failed to serialise manifest.".into())
    }

    fn write_manifest_file(mut file: fs::File, data: &[u8]) -> Result<(), String> {
        file.write_all(data)
            .map_err(|_| "Failed to write to manifest file.".into())
    }

    #[cfg(target_os = "windows")]
    fn register_regkey(browser: &Browser, json_path: &Path) -> Result<(), String> {
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
}

pub fn install_manifest<A, Installer: InstallerEffects<A>>(
    browser: &Browser,
) -> Result<PathBuf, A> {
    let manifest_path = Installer::get_manifest_path(&browser)?;
    Installer::create_manifest_dir(&manifest_path)?;

    let exe_path = Installer::get_exe_path()?;

    let filename = NAME.to_owned() + ".json";
    let full_write_path = manifest_path.join(filename);
    let file = Installer::create_manifest_file(&full_write_path)?;

    let manifest = Installer::generate_manifest(&browser, &exe_path)?;
    Installer::write_manifest_file(file, manifest.as_bytes())?;

    #[cfg(target_os = "windows")]
    B::register_regkey(&browser, &full_write_path)?;

    Ok(full_write_path)
}
