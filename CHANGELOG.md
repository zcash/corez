# Changelog
All notable changes to this library will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this library adheres to Rust's notion of
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `Read` and `Write` traits with safe `no_std` implementations.
- `Cursor<T>` for in-memory buffer I/O.
- `Error` and `ErrorKind` types compatible with `std::io`.
- Feature flags: `std` (default, re-exports `std::io`), `alloc`.
