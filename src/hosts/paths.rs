use std::path::PathBuf;

#[derive(Debug)]
pub enum Browser {
    Chrome,
    Chromium,
    Brave,
    Firefox,
}

#[derive(Debug)]
pub enum OsType {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

pub fn get_os_type() -> OsType {
    if cfg!(target_os = "linux") {
        OsType::Linux
    } else if cfg!(target_os = "macos") {
        OsType::MacOS
    } else if cfg!(target_os = "windows") {
        OsType::Windows
    } else {
        OsType::Unknown
    }
}

pub fn get_host_path(browser: &Browser) -> Result<PathBuf, &'static str> {
    let os_type = get_os_type();

    let home_dir = dirs::home_dir().ok_or("Failed to determine path to home directory.")?;
    let nm_dir_from_home = match (os_type, browser) {
        (OsType::Linux, Browser::Chrome) => Ok(".config/google-chrome/NativeMessagingHosts/"),
        (OsType::Linux, Browser::Chromium) => Ok(".config/chromium/NativeMessagingHosts/"),
        (OsType::Linux, Browser::Brave) => {
            Ok(".config/BraveSoftware/Brave-Browser/NativeMessagingHosts/")
        }
        (OsType::Linux, Browser::Firefox) => Ok(".mozilla/native-messaging-hosts/"),
        (OsType::MacOS, Browser::Chrome) => {
            Ok("Library/Application Support/Google/Chrome/NativeMessagingHosts/")
        }
        (OsType::MacOS, Browser::Chromium) => {
            Ok("Library/Application Support/Chromium/NativeMessagingHosts/")
        }
        (OsType::MacOS, Browser::Brave) => {
            Ok("Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/")
        }
        (OsType::MacOS, Browser::Firefox) => {
            Ok("Library/Application Support/Mozilla/NativeMessagingHosts/")
        }
        (OsType::Windows, Browser::Firefox) => Ok(r"AppData\Roaming\Mozilla\NativeMessagingHosts\"),
        (OsType::Windows, _) => Err("Windows is not yet fully supported."),
        _ => Err("Unrecognised operating system."),
    }?;

    Ok(home_dir.join(nm_dir_from_home))
}
