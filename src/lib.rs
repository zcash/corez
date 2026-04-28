//! *Safe, `no_std`-compatible I/O traits for the Zcash ecosystem.*
//!
//! When the `std` feature is enabled (the default), this crate re-exports
//! types from [`std::io`] directly. In `no_std` mode, it provides minimal
//! safe implementations of [`io::Read`], [`io::Write`], [`io::Cursor`],
//! [`io::Error`], and [`io::ErrorKind`].
//!
//! # Usage
//!
//! ```
//! use corez::io::{Read, Write};
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
// `doc(auto_cfg)` covers method-level `#[cfg(...)]` annotations. Cfgs on
// impl blocks (and other "container" items) currently need an explicit
// `#[cfg_attr(docsrs, doc(cfg(...)))]` as a workaround for
// https://github.com/rust-lang/rust/issues/150268 ; remove those when
// that is fixed.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, doc(auto_cfg))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", doc))]
extern crate std;

/// I/O traits, types, and error handling.
///
/// When the `std` feature is enabled, this module re-exports from
/// [`std::io`]. Otherwise it provides safe `no_std` implementations.
#[cfg(feature = "std")]
pub use std::io;

#[cfg(not(feature = "std"))]
#[cfg_attr(docsrs, doc(cfg(all())))]
pub mod io;
