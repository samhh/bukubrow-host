# Change Log
This project adheres to [Semantic Versioning](http://semver.org/).

## [5.1.0] - 2020-05-05
### Added
- Brave support.

## [5.0.0] - 2019-06-15
### Added
- Support for very large Buku databases that serialise to over 1MB in size.

### Changed
- Update WebExtension communication API to always operate on vectors of elements.

## [4.0.1] - 2019-05-07
### Changed
- Fix detection of operating system that failed on some Linux distros.
- Remove all panics. If something goes awry, make every attempt to communicate this to the consumer via stdout or over native messaging.

## [4.0.0] - 2019-05-04
### Added
- Self-installation of browser hosts, and removal of any external scripts or data files.
- Flags to list all bookmarks to stdout, and open bookmarks in browser by ID.

### Changed
- Host/binary moved to its own repo.
- The version has been significantly bumped to allow for the [WebExtension](https://github.com/SamHH/bukubrow-host) to piggyback off of it.
