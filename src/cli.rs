use crate::buku::types::BookmarkId;
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
    // Nota bene these strings determine the ordering of the help message
    let chrome_arg = "install-chrome";
    let chromium_arg = "install-chromium";
    let firefox_arg = "install-firefox";
    let brave_arg = "install-brave";
    let vivaldi_arg = "install-vivaldi";
    let edge_arg = "install-edge";
    let list_arg = "list";
    let open_arg = "open";

    let matches =
        App::new(crate_name!())
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
            .arg(Arg::new(edge_arg).long("--install-edge").about(
                "Install the native messaging host for Microsoft Edge (Dev channel on Linux)",
            ))
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

    let install_chrome = matches.is_present(chrome_arg);
    let install_chromium = matches.is_present(chromium_arg);
    let install_firefox = matches.is_present(firefox_arg);
    let install_brave = matches.is_present(brave_arg);
    let install_vivaldi = matches.is_present(vivaldi_arg);
    let install_edge = matches.is_present(edge_arg);
    let list_bookmarks = matches.is_present(list_arg);
    let open_bookmark_ids = matches.values_of(open_arg);

    let mut args = Vec::with_capacity(7);

    if install_chrome {
        args.push(Argument::InstallBrowserHost(Browser::Chrome));
    }
    if install_chromium {
        args.push(Argument::InstallBrowserHost(Browser::Chromium));
    }
    if install_firefox {
        args.push(Argument::InstallBrowserHost(Browser::Firefox));
    }
    if install_brave {
        args.push(Argument::InstallBrowserHost(Browser::Brave));
    }
    if install_vivaldi {
        args.push(Argument::InstallBrowserHost(Browser::Vivaldi));
    }
    if install_edge {
        args.push(Argument::InstallBrowserHost(Browser::Edge));
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
