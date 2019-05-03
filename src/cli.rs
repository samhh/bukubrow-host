use crate::hosts::paths::Browser;
use clap::{crate_authors, crate_name, crate_version, App, Arg, Error};

pub enum StdoutArg {
    ListBookmarks,
}

pub enum Argument {
    InstallBrowserHost(Browser),
    PrintStdout(StdoutArg),
}

/// Initialises the CLI interface and determines if the user explicitly passed
/// any known flags. An Err value denotes a parsing error, most likely meaning
/// unrecognised flag(s) were passed. This includes help and version flags
/// which must be handled outside this function.
pub fn init() -> Result<Option<Vec<Argument>>, Error> {
    let chrome_arg = "chrome";
    let chromium_arg = "chromium";
    let firefox_arg = "firefox";
    let list_arg = "list";

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bukubrow native messaging host installer")
        .arg(
            Arg::with_name(chrome_arg)
                .long("--install-chrome")
                .help("Install the native messaging host for Chrome."),
        )
        .arg(
            Arg::with_name(chromium_arg)
                .long("--install-chromium")
                .help("Install the native messaging host for Chromium."),
        )
        .arg(
            Arg::with_name(firefox_arg)
                .long("--install-firefox")
                .help("Install the native messaging host for Firefox."),
        )
        .arg(
            Arg::with_name(list_arg)
                .short("-l")
                .long("--list")
                .help("Print all bookmark in a list to stdout."),
        )
        .get_matches_safe()?;

    let install_chrome = matches.is_present(chrome_arg);
    let install_chromium = matches.is_present(chromium_arg);
    let install_firefox = matches.is_present(firefox_arg);
    let list_bookmarks = matches.is_present(list_arg);

    let mut args = Vec::with_capacity(4);

    if install_chrome {
        args.push(Argument::InstallBrowserHost(Browser::Chrome));
    }
    if install_chromium {
        args.push(Argument::InstallBrowserHost(Browser::Chromium));
    }
    if install_firefox {
        args.push(Argument::InstallBrowserHost(Browser::Firefox));
    }
    if list_bookmarks {
        args.push(Argument::PrintStdout(StdoutArg::ListBookmarks));
    }

    Ok(if args.is_empty() {
        None
    } else {
        Some(args)
    })
}
