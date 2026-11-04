# Changelog
All notable changes to this library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this library adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Without the `std` feature, `Read::read_exact` now fails in the same way as
  `std::io::Read::read_exact` when the input ends before the buffer is full:
  - the error message is "failed to fill whole buffer";
  - `<&[u8] as Read>::read_exact` consumes the remaining input;
  - `<Cursor<_> as Read>::read_exact` sets the position to the length of the
    buffer.
- Without the `std` feature, `Write::write_all` now fails in the same way as
  `std::io::Write::write_all` when the writer accepts no more bytes:
  - the error message is "failed to write whole buffer";
  - `<&mut [u8] as Write>::write_all` writes the part of the buffer that fits
    and consumes the slice.

### Fixed
- docs.rs now builds documentation with `--no-default-features --features alloc`,
  so the published docs describe the `corez::io` module's own API rather than
  the `std::io` re-export. (#5)

## [0.1.1] - 2026-04-17

### Changed
- `corez` now uses `edition = "2021"` instead of `edition = "2024"`.

## [0.1.0] - 2026-04-17

Initial Release
