#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod buku;
mod cli;
mod config;
mod manifest;
mod native_messaging;
mod server;

use crate::buku::database::{BukuDatabase, SqliteDatabase};
use crate::buku::utils::get_db_path;
use crate::cli::{exit_with_stdout_err, Argument};
use crate::manifest::installer::install_manifest;
use crate::native_messaging::NativeMessagingError;
use crate::server::{map_init_err_friendly_msg, InitError, Server};
use clap::error::ErrorKind;
use std::path::PathBuf;

fn main() {
    let db = get_db_path()
        .map_err(|_| InitError::FailedToLocateBukuDatabase)
        .and_then(|path| {
            SqliteDatabase::new(&path).map_err(|_| InitError::FailedToAccessBukuDatabase)
        });

    // Native messaging can provide its own arguments we don't care about, so
    // ignore any unrecognised arguments
    let recognised_arg = cli::init().unwrap_or_else(|err| match err.kind() {
        ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => err.exit(),
        _ => None,
    });

    // Only continue to native messaging if no recognised flags are found
    if let Some(arg) = recognised_arg {
        match db {
            Ok(db) => match arg {
                Argument::InstallBrowserHost(browser, path) => {
                    let installed = install_manifest(&browser, path.map(PathBuf::from));

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
                Argument::ListBookmarks => match db.get_all_bookmarks() {
                    Ok(bms) => {
                        for bm in bms {
                            println!("{} {}", bm.id, bm.metadata);
                        }
                    }
                    Err(_) => {
                        exit_with_stdout_err("Failed to fetch bookmarks from database.");
                    }
                },
                Argument::OpenBookmarks(ids) => match db.get_bookmarks_by_id(ids) {
                    Ok(bms) => {
                        for bm in bms {
                            if webbrowser::open(&bm.url).is_err() {
                                exit_with_stdout_err("Failed to open bookmark in web browser.");
                            }
                        }
                    }
                    Err(_) => {
                        exit_with_stdout_err("Failed to fetch selected bookmarks from database.");
                    }
                },
            },
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
