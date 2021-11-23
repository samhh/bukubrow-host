use platforms::target::{OS, TARGET_OS};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Browser {
    Chrome,
    Chromium,
    Brave,
    Firefox,
    LibreWolf,
    Vivaldi,
    Edge,
}

pub fn get_manifest_path(browser: &Browser) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to determine path to home directory.")?;

    let nm_dir_from_home = match TARGET_OS {
        OS::Linux | OS::OpenBSD | OS::FreeBSD => match browser {
            Browser::Chrome => Ok(".config/google-chrome/NativeMessagingHosts/"),
            Browser::Chromium => Ok(".config/chromium/NativeMessagingHosts/"),
            Browser::Brave => Ok(".config/BraveSoftware/Brave-Browser/NativeMessagingHosts/"),
            Browser::Firefox => Ok(".mozilla/native-messaging-hosts/"),
            Browser::LibreWolf => Ok(".librewolf/native-messaging-hosts/"),
            Browser::Vivaldi => Ok(".config/vivaldi/NativeMessagingHosts/"),
            Browser::Edge => Ok(".config/microsoft-edge-dev/NativeMessagingHosts/"),
        },
        OS::MacOS => match browser {
            Browser::Chrome => {
                Ok("Library/Application Support/Google/Chrome/NativeMessagingHosts/")
            }
            Browser::Chromium => Ok("Library/Application Support/Chromium/NativeMessagingHosts/"),
            Browser::Brave => {
                Ok("Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/")
            }
            Browser::Firefox => Ok("Library/Application Support/Mozilla/NativeMessagingHosts/"),
            Browser::LibreWolf => Ok("Library/Application Support/LibreWolf/NativeMessagingHosts/"),
            Browser::Vivaldi => Ok("Library/Application Support/Vivaldi/NativeMessagingHosts/"),
            Browser::Edge => Ok("Library/Microsoft/Edge/NativeMessagingHosts/"),
        },
        OS::Windows => match browser {
            // LibreWolf and Firefox share the same registry key, so they should also share the same directory
            Browser::Firefox | Browser::LibreWolf => Ok(r"AppData\Roaming\Mozilla\NativeMessagingHosts\"),
            browser => Err(format!("{:?} is not yet supported on Windows.", browser)),
        },
        os => Err(format!("Platform \"{}\" is not yet supported.", os)),
    }?;

    Ok(home_dir.join(nm_dir_from_home))
}

#[cfg(target_os = "windows")]
pub fn get_regkey_path(browser: &Browser) -> Option<&'static str> {
    match browser {
        Browser::Firefox | Browser::LibreWolf => Some(r"Software\Mozilla\NativeMessagingHosts"),
        _ => None,
    }
}
