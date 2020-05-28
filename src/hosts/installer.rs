use super::paths::{get_host_path, get_os_type, Browser, OsType};
use super::targets::chrome::ChromeHost;
use super::targets::firefox::FirefoxHost;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const NM_REGKEY: &'static str = "com.samhh.bukubrow";
const NM_FILENAME: &'static str = "com.samhh.bukubrow.json";

pub fn install_host(browser: &Browser) -> Result<PathBuf, &'static str> {
    // Create native messaging path if it doesn't already exist
    let host_path = get_host_path(&browser)?;
    fs::create_dir_all(&host_path).map_err(|_| "Failed to create native messaging directory.")?;

    // Determine path of self/executable
    let exe_err_str = "Could not determine location of Bukubrow executable.";
    let exe_path = std::env::current_exe()
        .map_err(|_| exe_err_str)
        .and_then(|path| path.into_os_string().into_string().map_err(|_| exe_err_str))?;

    // Create JSON file
    let full_write_path = PathBuf::from(host_path).join(NM_FILENAME);
    let mut file =
        fs::File::create(&full_write_path).map_err(|_| "Failed to create browser host file.")?;

    // Write to created file
    match browser {
        Browser::Chrome | Browser::Chromium | Browser::Brave => file.write_all(
            &serde_json::to_string(&ChromeHost::new(exe_path))
                .map_err(|_| "Failed to serialise Chrome/Chromium/Brave browser host.")?
                .as_bytes(),
        ),
        Browser::Firefox => file.write_all(
            &serde_json::to_string(&FirefoxHost::new(exe_path))
                .map_err(|_| "Failed to serialise Firefox browser host.")?
                .as_bytes(),
        ),
    }
    .map_err(|_| "Failed to write to browser host file.")?;

    let os_type = get_os_type();
    match (os_type, browser) {
        (OsType::Windows, Browser::Firefox) => register_firefox(&full_write_path)?,
        _ => (),
    };

    Ok(full_write_path)
}

fn register_firefox(json_path: &PathBuf) -> Result<(), &'static str> {
    let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
    let path = PathBuf::from(r"Software\Mozilla\NativeMessagingHosts").join(NM_REGKEY);
    let (key, _) = hkcu
        .create_subkey(&path)
        .map_err(|_| "Failed to create registry entry.")?;

    key.set_value("", &json_path.to_string_lossy().into_owned())
        .map_err(|_| "Failed to set registry entry.")?;

    Ok(())
}
