Bukubrow Host
===

[Bukubrow](https://github.com/SamHH/bukubrow-webext) is a WebExtension for [Buku](https://github.com/jarun/Buku), a command-line bookmark manager. This is the corresponding host that facilitates interfacing with the Buku database via [native messaging](https://developer.chrome.com/extensions/nativeMessaging).

## Prerequisites

- Buku
- Bukubrow WebExtension
- _If building the host_:
	- Rust / Cargo

## Installation

Installing the host and registering it with your browser is required to allow the browser extension to talk to Buku.

If you've downloaded a host zip from the [releases page](https://github.com/samhh/bukubrow-host/releases), skip to step 3.

1. Clone the repo.
2. Run `make build-linux-x64` (Linux) or `make build-darwin-x64` (macOS). Note that you'll need your target platform installed and configured with Cargo. Your zip file will be located within the `./release/` directory.
3. Extract the zip file and move the executable to a suitable location, for example `/usr/local/bin/`.
4. Install the host file for your browser via the executable; options can be viewed with `bukubrow --help`.

## Contributing

The host is written in Rust stable (1.34.1 at time of writing). The messages it expects to receive from the WebExtension follow a faux HTTP format; for instance, to get all the bookmarks, you pass it a JSON object of the following format: `{ method: 'GET' }`.
