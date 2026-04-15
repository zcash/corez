//! *Safe, `no_std`-compatible I/O traits for the Zcash ecosystem.*
//!
//! When the `std` feature is enabled (the default), this crate re-exports
//! types from `std::io` directly. In `no_std` mode, it provides minimal
//! safe implementations of [`Read`], [`Write`], [`Cursor`], [`Error`], and
//! [`ErrorKind`].

#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// --- std path: re-export everything from std::io ---
#[cfg(feature = "std")]
pub use std::io::{Cursor, Error, ErrorKind, Read, Result, Write};

// --- no_std path: our own safe implementations ---
#[cfg(not(feature = "std"))]
mod error;

#[cfg(not(feature = "std"))]
pub use error::{Error, ErrorKind};

#[cfg(not(feature = "std"))]
/// A specialized [`Result`](core::result::Result) type for I/O operations.
pub type Result<T> = core::result::Result<T, Error>;

#[cfg(not(feature = "std"))]
mod traits;

#[cfg(not(feature = "std"))]
pub use traits::{Read, Write};

#[cfg(not(feature = "std"))]
mod impls;

#[cfg(not(feature = "std"))]
mod cursor;

#[cfg(not(feature = "std"))]
pub use cursor::Cursor;
