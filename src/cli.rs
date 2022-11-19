use crate::buku::types::BookmarkId;
use crate::manifest::paths::Browser;
use clap::{crate_authors, crate_name, crate_version, Command, Arg, Error as ClapError};

pub enum Argument {
    /// The second piece of data is an optional custom installation dir.
    InstallBrowserHost(Browser, Option<String>),
    ListBookmarks,
    OpenBookmarks(Vec<BookmarkId>),
}

/// Initialises the CLI interface and determines if the user explicitly passed
/// any known flags. An Err value denotes a parsing error, most likely meaning
/// unrecognised flag(s) were passed. This includes help and version flags
/// which must be handled outside this function.
pub fn init() -> Result<Option<Argument>, ClapError> {
    // Nota bene these strings determine the ordering of the help message
    let chrome_arg = "install-chrome";
    let chromium_arg = "install-chromium";
    let firefox_arg = "install-firefox";
    let librewolf_arg = "install-librewolf";
    let brave_arg = "install-brave";
    let vivaldi_arg = "install-vivaldi";
    let edge_arg = "install-edge";
    let dir_arg = "install-dir";
    let list_arg = "list";
    let open_arg = "open";

    let matches = Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Bukubrow native messaging host installer")
        .arg(
            Arg::new(chrome_arg)
                .long("install-chrome")
                .num_args(0)
                .help("Install the native messaging host for Chrome"),
        )
        .arg(
            Arg::new(chromium_arg)
                .long("install-chromium")
                .num_args(0)
                .help("Install the native messaging host for Chromium"),
        )
        .arg(
            Arg::new(firefox_arg)
                .long("install-firefox")
                .num_args(0)
                .help("Install the native messaging host for Firefox"),
        )
        .arg(
            Arg::new(librewolf_arg)
                .long("install-librewolf")
                .num_args(0)
                .help("Install the native messaging host for LibreWolf"),
        )
        .arg(
            Arg::new(brave_arg)
                .long("install-brave")
                .num_args(0)
                .help("Install the native messaging host for Brave"),
        )
        .arg(
            Arg::new(vivaldi_arg)
                .long("install-vivaldi")
                .num_args(0)
                .help("Install the native messaging host for Vivaldi"),
        )
        .arg(
            Arg::new(edge_arg)
                .long("install-edge")
                .num_args(0)
                .help("Install the native messaging host for Edge"),
        )
        .arg(
            Arg::new(dir_arg)
                .long("install-dir")
                .help("Specify a custom manifest installation directory")
                .value_name("DIR")
                .num_args(1),
        )
        .arg(
            Arg::new(list_arg)
                .short('l')
                .long("list")
                .num_args(0)
                .help("Print all bookmarks in a list to stdout"),
        )
        .arg(
            Arg::new(open_arg)
                .short('o')
                .long("open")
                .help("Open bookmark(s) in the browser by ID")
                .value_delimiter(',')
                .value_name("ID[,ID]")
                .value_parser(clap::value_parser!(u32))
                .num_args(1),
        )
        .try_get_matches()?;

    if let Some(vals) = matches.get_many::<u32>(open_arg) {
        let ids = vals.copied().collect();

        return Ok(Some(Argument::OpenBookmarks(ids)));
    }

    if matches.contains_id(list_arg) {
        return Ok(Some(Argument::ListBookmarks));
    }

    let dir = matches.get_one::<String>(dir_arg).map(|x| x.clone());
    if matches.contains_id(chrome_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Chrome, dir)));
    }
    if matches.contains_id(chromium_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Chromium, dir)));
    }
    if matches.contains_id(firefox_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Firefox, dir)));
    }
    if matches.contains_id(librewolf_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::LibreWolf, dir)));
    }
    if matches.contains_id(brave_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Brave, dir)));
    }
    if matches.contains_id(vivaldi_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Vivaldi, dir)));
    }
    if matches.contains_id(edge_arg) {
        return Ok(Some(Argument::InstallBrowserHost(Browser::Edge, dir)));
    }

    Ok(None)
}

pub fn exit_with_stdout_err<T: std::fmt::Display>(msg: T) -> ! {
    println!("{}", msg);
    std::process::exit(1);
}
