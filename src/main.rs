#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod buku;
mod cli;
mod database;
mod hosts;
mod server;

use crate::cli::{exit_with_stdout_err, Argument, CliError};
use crate::database::SqliteDatabase;
use crate::hosts::installer::install_host;
use crate::server::Server;
use clap::ErrorKind;

fn main() {
    // Native messaging can provide its own arguments we don't care about, so
    // ignore any unrecognised arguments
    let args_maybe = cli::init().unwrap_or_else(|err| match err {
        CliError::Clap(clap_err) => match clap_err.kind {
            ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed => clap_err.exit(),
            _ => None,
        },
        CliError::BookmarkIdsParseFailed => {
            exit_with_stdout_err("Failed to parse bookmark ID(s).");
        }
    });

    // Only continue to native messaging if no recognised flags are found
    if let Some(args) = args_maybe {
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
                    let path = buku::get_db_path().unwrap();
                    let db = SqliteDatabase::new(&path).unwrap();
                    let bms = db.get_all_bookmarks().unwrap();

                    for bm in bms {
                        if let Some(id) = bm.id {
                            println!("{} {}", id, bm.metadata);
                        }
                    }
                }
                Argument::OpenBookmarks(ids) => {
                    let path = buku::get_db_path().unwrap();
                    let db = SqliteDatabase::new(&path).unwrap();
                    let bms = db.get_bookmarks_by_id(ids).unwrap();

                    for bm in bms {
                        webbrowser::open(&bm.url).unwrap();
                    }
                }
            }
        }

        std::process::exit(0);
    }

    // No installation arguments supplied, proceed with native messaging
    let path = buku::get_db_path().unwrap();
    let db = SqliteDatabase::new(&path).unwrap();
    let server = Server::new(db);

    server.listen();
}
