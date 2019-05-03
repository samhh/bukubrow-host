#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

mod buku;
mod cli;
mod database;
mod hosts;
mod server;

use crate::cli::{Argument, StdoutArg};
use crate::database::SqliteDatabase;
use crate::hosts::installer::install_host;
use crate::server::Server;
use clap::ErrorKind;

fn main() {
    // Native messaging can provide its own arguments we don't care about, so
    // ignore any unrecognised arguments
    let args_maybe = cli::init().unwrap_or_else(|err| match err.kind {
        ErrorKind::HelpDisplayed | ErrorKind::VersionDisplayed => err.exit(),
        _ => None,
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
                            println!("Failed to install host for {:?}:\n\t{}", &browser, err,);

                            std::process::exit(1);
                        }
                    };
                },
                Argument::PrintStdout(arg) => {
                    match arg {
                        StdoutArg::ListBookmarks => {
                            let path = buku::get_db_path().unwrap();
                            let db = SqliteDatabase::new(&path).unwrap();
                            let bms = db.get_bookmarks().unwrap();

                            for bm in bms {
                                if let Some(id) = bm.id {
                                    println!("{} {}", id, bm.metadata);
                                }
                            }
                        }
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
