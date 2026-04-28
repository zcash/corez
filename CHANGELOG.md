# Changelog
All notable changes to this library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this library adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed
- docs.rs now builds documentation with `--no-default-features --features alloc`,
  so the published docs describe the `corez::io` module's own API rather than
  the `std::io` re-export. (#5)

## [0.1.1] - 2026-04-17

### Changed
- `corez` now uses `edition = "2021"` instead of `edition = "2024"`.

## [0.1.0] - 2026-04-17

Initial Release
