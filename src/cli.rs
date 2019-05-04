use crate::database::BookmarkId;
use crate::hosts::paths::Browser;
use clap::{crate_authors, crate_name, crate_version, App, Arg, Error as ClapError};

pub enum Argument {
    InstallBrowserHost(Browser),
    ListBookmarks,
    OpenBookmarks(Vec<BookmarkId>),
}

#[derive(Debug)]
pub enum CliError {
    Clap(ClapError),
    BookmarkIdsParseFailed,
}

/// Initialises the CLI interface and determines if the user explicitly passed
/// any known flags. An Err value denotes a parsing error, most likely meaning
/// unrecognised flag(s) were passed. This includes help and version flags
/// which must be handled outside this function.
pub fn init() -> Result<Option<Vec<Argument>>, CliError> {
    let chrome_arg = "chrome";
    let chromium_arg = "chromium";
    let firefox_arg = "firefox";
    let list_arg = "list";
    let open_arg = "open";

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
                .help("Print all bookmarks in a list to stdout."),
        )
        .arg(
            Arg::with_name(open_arg)
                .short("-o")
                .long("--open")
                .help("Open bookmark(s) in the browser by ID.")
                .takes_value(true)
                .value_delimiter(",")
                .value_name("ID[,ID]"),
        )
        .get_matches_safe()
        .map_err(|e| CliError::Clap(e))?;

    let install_chrome = matches.is_present(chrome_arg);
    let install_chromium = matches.is_present(chromium_arg);
    let install_firefox = matches.is_present(firefox_arg);
    let list_bookmarks = matches.is_present(list_arg);
    let open_bookmark_ids = matches.values_of(open_arg);

    let mut args = Vec::with_capacity(5);

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
        args.push(Argument::ListBookmarks);
    }
    if let Some(vals) = open_bookmark_ids {
        let mut ids = Vec::with_capacity(vals.len());

        for val in vals {
            ids.push(val.parse().map_err(|_| CliError::BookmarkIdsParseFailed)?);
        }

        args.push(Argument::OpenBookmarks(ids));
    }

    Ok(if args.is_empty() { None } else { Some(args) })
}

pub fn exit_with_stdout_err<T: std::fmt::Display>(msg: T) -> ! {
    println!("{}", msg);
    std::process::exit(1);
}
