use platforms::target::{OS, TARGET_OS};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Browser {
    Chrome,
    Chromium,
    Brave,
    Firefox,
}

pub fn get_host_path(browser: &Browser) -> Result<PathBuf, String> {
    let home_dir = dirs::home_dir().ok_or("Failed to determine path to home directory.")?;

    let nm_dir_from_home = match TARGET_OS {
        OS::Linux | OS::OpenBSD | OS::FreeBSD => match browser {
            Browser::Chrome => Ok(".config/google-chrome/NativeMessagingHosts/"),
            Browser::Chromium => Ok(".config/chromium/NativeMessagingHosts/"),
            Browser::Brave => Ok(".config/BraveSoftware/Brave-Browser/NativeMessagingHosts/"),
            Browser::Firefox => Ok(".mozilla/native-messaging-hosts/"),
        },
        OS::MacOS => match browser {
            Browser::Chrome => Ok("Library/Application Support/Google/Chrome/NativeMessagingHosts/"),
            Browser::Chromium => Ok("Library/Application Support/Chromium/NativeMessagingHosts/"),
            Browser::Brave => Ok("Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/"),
            Browser::Firefox => Ok("Library/Application Support/Mozilla/NativeMessagingHosts/"),
        },
        OS::Windows => match browser {
            Browser::Firefox => Ok(r"AppData\Roaming\Mozilla\NativeMessagingHosts\"),
            browser => Err(format!("{:?} is not yet supported on Windows.", browser)),
        },
        os => Err(format!("Platform \"{}\" is not yet supported.", os)),
    }?;

    Ok(home_dir.join(nm_dir_from_home))
}
