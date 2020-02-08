#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod buku;
mod cli;
mod hosts;
mod matching;
mod native_messaging;
mod server;

use crate::buku::database::{BukuDatabase, SqliteDatabase};
use crate::buku::utils::get_db_path;
use crate::cli::{exit_with_stdout_err, Argument, CliError};
use crate::hosts::installer::install_host;
use crate::native_messaging::NativeMessagingError;
use crate::server::{map_init_err_friendly_msg, InitError, Server};
use clap::ErrorKind;

fn main() {
    let db = get_db_path()
        .map_err(|_| InitError::FailedToLocateBukuDatabase)
        .and_then(|path| {
            SqliteDatabase::new(&path).map_err(|_| InitError::FailedToAccessBukuDatabase)
        });

    // Native messaging can provide its own arguments we don't care about, so
    // ignore any unrecognised arguments
    let recognised_args = cli::init().unwrap_or_else(|err| match err {
        CliError::Clap(clap_err) => match clap_err.kind {
            ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed => clap_err.exit(),
            _ => None,
        },
        CliError::BookmarkIdsParseFailed => {
            exit_with_stdout_err("Failed to parse bookmark ID(s).");
        }
    });

    // Only continue to native messaging if no recognised flags are found
    if let Some(args) = recognised_args {
        match db {
            Ok(db) => {
                for arg in args {
                    match arg {
                        Argument::InstallBrowserHost(browser) => {
                            let installed = install_host(&browser);

                            match installed {
                                Ok(path) => {
                                    println!(
                                        "Successfully installed host for {:?} to:\n\t{:?}",
                                        &browser, path,
                                    );
                                }
                                Err(err) => {
                                    exit_with_stdout_err(format!(
                                        "Failed to install host for {:?}:\n\t{}",
                                        &browser, err
                                    ));
                                }
                            };
                        }
                        Argument::ListBookmarks => {
                            for bm in db.get_all_bookmarks() {
                                println!("{} {}", bm.id, bm.metadata);
                            }
                        },
                        Argument::OpenBookmarks(ids) => {
                            for bm in db.get_bookmarks_by_id(ids) {
                                if let Err(_) = webbrowser::open(&bm.url) {
                                    exit_with_stdout_err(
                                        "Failed to open bookmark in web browser.",
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(err) => {
                exit_with_stdout_err(map_init_err_friendly_msg(&err));
            }
        }

        std::process::exit(0);
    }

    // No installation arguments supplied, proceed with native messaging. Do not
    // exit if cannot find or access Buku database, instead allow server to
    // communicate that. This is an asynchronous call.
    let res = Server::new(db).listen();

    match res {
        Ok(_) | Err(NativeMessagingError::NoMoreInput) => std::process::exit(0),
        _ => std::process::exit(1),
    }
}
