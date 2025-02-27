# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.2.9] - 2025-02-02

### Fixed
- Fix info text for channels

## [0.2.8] - 2025-01-26

### Fixed
- Improve `README.md` text

## [0.2.7] - 2025-01-22

### Fixed
- Improve `--help` and `README.md` texts

## [0.2.6] - 2025-01-20

### Changed
- Rename repository to `nps`

## [0.2.5] - 2025-01-18

## [0.2.4] - 2025-01-18

### Added
- Non-debug messages for better feedback
- Quiet option `-q/--quiet` to suppress non-debug messages

### Fixed
- Run instructions for channel systems

## [0.2.3] - 2025-01-17

### Added
- Automated releases

### Deprecated
- Installation via 'defaultPackage', use 'packages.\<system\>.default' instead

### Fixed
- Update documentation
- Improve logging
- Minor bugfixes

## [0.2.2] - 2025-01-10

### Added
- Automated tests via GitHub Actions
- Releases made with cargo-release
- Various pre-commit checks
- Version tags for development builds
- More tests

## [0.2.1] - 2025-01-06

### Fixed
- Cache tests
- Be more fine-grained with cache refresh errors

## [0.2.0] - 2025-01-05

### Added
- Optionally use the nix system registry flake as a source
  - Use option `-e`/`--experimental`
- Update documentation
- Show environment variable settings in `-h`/`--help`
- `-d`/`--debug` for debug info, use multiple times for increased verbosity
- `-i`/`--ignore-case` option

### Changed
- Fancier `-h`/`--help` with colors
- Supplying flags with arguments require the use of "="
  - `-f=true`
  - Or use the default by omitting the flag argument `-f`
- Saner defaults
  - Inverted `-f`/`--flip`, show most important matches below
  - Case-insensitive search by default
- Use `--help` instead of `-l` for long help
- Version flag changed to `-V`/`--version`

### Removed
- Settings for file names

### Fixed
- Inconsistent use of empty lines separating matches
- Duplicated package matches
- Much faster now!

## [0.1.6] - 2023-01-08

### Fixed
- Use GNU versions of awk and sed

## [0.1.5] - 2022-12-20

### Fixed
- Create cache directory when needed

## [0.1.4] - 2022-12-20

### Fixed
- Print the correct version number

## [0.1.3] - 2022-12-20

### Fixed
- Dependencies are now hidden and won't clash with existing packages anymore

## [0.1.2] - 2022-12-18

### Fixed
- Remove debug print

## [0.1.1] - 2022-12-18

### Fixed
- Document "flip" option in README.md

## [0.1.0] - 2022-12-18

### Added
- "Flip" option: reverse result order for better visibility of the most relevant matches

## [0.0.1] - 2022-12-18

### Added
- This changelog

### Changed

- Versioning now adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

<!-- next-url -->
[Unreleased]: https://github.com/OleMussmann/nps/compare/v0.2.9...development
[0.2.9]: https://github.com/OleMussmann/nps/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/OleMussmann/nps/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/OleMussmann/nps/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/OleMussmann/nps/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/OleMussmann/nps/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/OleMussmann/nps/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/OleMussmann/nps/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/OleMussmann/nps/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/OleMussmann/nps/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/OleMussmann/nps/compare/v0.1.6...v0.2.0
[0.1.6]: https://github.com/OleMussmann/nps/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/OleMussmann/nps/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/OleMussmann/nps/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/OleMussmann/nps/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/OleMussmann/nps/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/OleMussmann/nps/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/OleMussmann/nps/compare/v0.0.1...v0.1.0
[0.0.1]: https://github.com/OleMussmann/nps/releases/tag/v0.0.1
