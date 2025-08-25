# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.0] - 2025-08-25

### Changed
- **Improved**: derive `#[from]` now relaxed to allow `T: Display + Debug` instead of `T: Error`

## [2.2.2] - 2025-07-23

### Changed
- **Chore**: Added CHANGELOG badge and link, because people want to see it easily
- **Chore**: Renamed branch from master to main

## [2.2.1] - 2025-07-23

### Changed
- **Improved documentation**: Enhanced crate description and library docs / README

## [2.2.0] - 2025-07-23

### Added
- **Debug fallback support**: New `#[error(debug)]` attribute for automatic Display generation using Debug formatting
- Support for `#[error(debug)]` at enum type level to provide fallback for variants without explicit error messages
- Support for `#[error(debug)]` on individual enum variants and struct types
- Comprehensive validation to prevent conflicting attributes (`debug` with `display`, `fmt`, or `transparent`)
- Enhanced enum Display generation with proper Debug formatting for unit, tuple, and struct variants
- Added extensive test coverage for debug fallback functionality

### Changed
- Enhanced attribute parsing to recognize and validate `debug` attribute
- Updated enum validation logic to allow variants without explicit display attributes when enum has `#[error(debug)]` fallback
- Improved Display implementation generation for enums with mixed explicit and debug-fallback variants

## [2.1.0] - 2025-07-23

### Added
- Auto-generate `From<T>` implementations for `Box<T>` fields with `#[from]` attribute
- Support for `Option<Box<T>>` fields with automatic boxing in `From<T>` implementations
- Added CHANGELOG + semantic versioning
- Got most lib docs to compile and synced with README using `cargo-readme`
- Added justfile with `preflight` version check before `publish`

### Changed
- Enhanced `from_impl` generation to detect `Box<T>` types and generate additional `From<T>` implementations
- Updated enum `from_impls` generation to handle multiple implementations per variant using `flat_map`

### Documentation
- Added documentation and examples for `Box<T>` field `From<T>` generation in README

## [2.0.13] - 2025-07-21

### Added
- `.location()` method for derived Error implementations
- Returns `Option<&'static Location<'static>>` when location field exists
- Support for mixed enum variants (some with location, some without)
- Comprehensive tests including `#[track_caller]` behavior validation

### Fixed
- Fixed references in documentation and code

## [2.0.12] - 2025-07-21

### Added
- **Initial fork of thiserror with `std::panic::Location` support**
- Auto-detection of `Location<'static>` fields in structs and enum variants
- Optional `#[location]` attribute for explicit field marking
- Automatic `From` impl generation with `#[track_caller]`
- Support for both named and tuple enum variants
- Location populated via `Location::caller()` during `From` conversion
- No `#[from]` attribute required on location fields

### Changed
- Rebranded from `thiserror` to `wherror` across all packages
- Updated all imports and references throughout codebase
- Added comprehensive README documentation for location feature

### Documentation
- Updated README with location feature examples and usage
- Added proper attribution to original thiserror and PR #291 authors

## Notes

This changelog starts from the point where wherror was forked from thiserror 2.0.12.
For the complete history of the original thiserror project, see the
[thiserror repository](https://github.com/dtolnay/thiserror).

The fork was based on thiserror 2.0.12 (commit 6c1cc96) and incorporates
`std::panic::Location` support for automatic call site location capture.

[Unreleased]: https://github.com/dra11y/wherror/compare/v2.0.13...HEAD
[2.0.13]: https://github.com/dra11y/wherror/compare/v2.0.12...v2.0.13
[2.0.12]: https://github.com/dra11y/wherror/releases/tag/v2.0.12
