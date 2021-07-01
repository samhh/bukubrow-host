Bukubrow Host
===

[Bukubrow](https://github.com/SamHH/bukubrow-webext) is a WebExtension for [Buku](https://github.com/jarun/Buku), a command-line bookmark manager. This is the corresponding host that facilitates interfacing with the Buku database via [native messaging](https://developer.chrome.com/extensions/nativeMessaging).

```
USAGE:
    bukubrow [FLAGS] [OPTIONS]

FLAGS:
    -h, --help                Prints help information
        --install-brave       Install the native messaging host for Brave
        --install-chrome      Install the native messaging host for Chrome
        --install-chromium    Install the native messaging host for Chromium
        --install-edge        Install the native messaging host for Edge
        --install-firefox     Install the native messaging host for Firefox
        --install-vivaldi     Install the native messaging host for Vivaldi
    -l, --list                Print all bookmarks in a list to stdout
    -V, --version             Prints version information

OPTIONS:
        --install-dir <dir>    Specify a custom manifest installation directory
    -o, --open <ID[,ID]>       Open bookmark(s) in the browser by ID
```

## Prerequisites

- Buku
- Bukubrow WebExtension
- _If building the host_:
	- Rust / Cargo

## Installation

Installing the host and registering it with your browser is required to allow the browser extension to talk to Buku.

The host is available from the following package managers:

- AUR: [bukubrow](https://aur.archlinux.org/packages/bukubrow/) (official) / [bukubrow-bin](https://aur.archlinux.org/packages/bukubrow-bin/) (official)
- Nix: [bukubrow](https://search.nixos.org/packages?channel=unstable&query=bukubrow)

If you've installed the host via a package manager, skip to step 4.

If you've downloaded the host from the [releases page](https://github.com/samhh/bukubrow-host/releases), skip to step 3.

1. Clone the repo.
2. Run `cargo build --release`. Note that you'll need your target platform installed and configured with Cargo. Your executable will be located at `target/release/bukubrow`.
3. Move the executable to a suitable location, for example `/usr/local/bin/`.
4. Install the host file for your browser via the executable, for example `bukubrow --install-firefox`.

Further options can be viewed with `bukubrow --help`.

## Contributing

The host is written in Rust stable. The messages it expects to receive from the WebExtension follow a faux HTTP format; for instance, to get all the bookmarks, you pass it a JSON object of the following format: `{ "method": "GET" }`.

