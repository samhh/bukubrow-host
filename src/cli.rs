use crate::buku::types::BookmarkId;
use crate::manifest::paths::Browser;
use clap::{crate_authors, crate_name, crate_version, App, Arg, Error as ClapError};

pub enum Argument {
    /// The second piece of data is an optional custom installation dir.
    InstallBrowserHost(Browser, Option<String>),
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
pub fn init() -> Result<Option<Argument>, CliError> {
    // Nota bene these strings determine the ordering of the help message
    let chrome_arg = "install-chrome";
    let chromium_arg = "install-chromium";
    let firefox_arg = "install-firefox";
    let brave_arg = "install-brave";
    let vivaldi_arg = "install-vivaldi";
    let edge_arg = "install-edge";
    let dir_arg = "install-dir";
    let list_arg = "list";
    let open_arg = "open";

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bukubrow native messaging host installer")
        .arg(
            Arg::new(chrome_arg)
                .long("--install-chrome")
                .about("Install the native messaging host for Chrome"),
        )
        .arg(
            Arg::new(chromium_arg)
                .long("--install-chromium")
                .about("Install the native messaging host for Chromium"),
        )
        .arg(
            Arg::new(firefox_arg)
                .long("--install-firefox")
                .about("Install the native messaging host for Firefox"),
        )
        .arg(
            Arg::new(brave_arg)
                .long("--install-brave")
                .about("Install the native messaging host for Brave"),
        )
        .arg(
            Arg::new(vivaldi_arg)
                .long("--install-vivaldi")
                .about("Install the native messaging host for Vivaldi"),
        )
        .arg(
            Arg::new(edge_arg)
                .long("--install-edge")
                .about("Install the native messaging host for Edge"),
        )
        .arg(
            Arg::new(dir_arg)
                .long("--install-dir")
                .about("Specify a custom manifest installation directory")
                .takes_value(true)
                .value_name("dir"),
        )
        .arg(
            Arg::new(list_arg)
                .short('l')
                .long("--list")
                .about("Print all bookmarks in a list to stdout"),
        )
        .arg(
            Arg::new(open_arg)
                .short('o')
                .long("--open")
                .about("Open bookmark(s) in the browser by ID")
                .takes_value(true)
                .value_delimiter(",")
                .value_name("ID[,ID]"),
        )
        .try_get_matches()
        .map_err(CliError::Clap)?;

    if let Some(vals) = matches.values_of(open_arg) {
        let mut ids = Vec::with_capacity(vals.len());

        for val in vals {
            ids.push(val.parse().map_err(|_| CliError::BookmarkIdsParseFailed)?);
        }

        return Ok(Some(Argument::OpenBookmarks(ids)));
    }

    if matches.is_present(list_arg) {
        return Ok(Some(Argument::ListBookmarks));
    }

    let dir = matches.value_of(dir_arg).map(String::from);
    if matches.is_present(chrome_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Chrome, dir)));
    }
    if matches.is_present(chromium_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Chromium, dir)));
    }
    if matches.is_present(firefox_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Firefox, dir)));
    }
    if matches.is_present(brave_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Brave, dir)));
    }
    if matches.is_present(vivaldi_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Vivaldi, dir)));
    }
    if matches.is_present(edge_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Edge, dir)));
    }

    Ok(None)
}

pub fn exit_with_stdout_err<T: std::fmt::Display>(msg: T) -> ! {
    println!("{}", msg);
    std::process::exit(1);
}
