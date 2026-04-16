use core::fmt;

/// Categories of I/O errors.
///
/// This is a reduced subset of [`std::io::ErrorKind`] containing only the
/// variants used by the Zcash ecosystem crates.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum ErrorKind {
    /// An entity was not found (e.g. a file or directory).
    NotFound,
    /// Data not valid for the operation being performed.
    InvalidData,
    /// A parameter was incorrect.
    InvalidInput,
    /// An operation could not be completed because a call to `write`
    /// returned `Ok(0)`.
    WriteZero,
    /// An operation could not be completed because an "end of file" was
    /// reached prematurely.
    UnexpectedEof,
    /// The operation was interrupted and can typically be retried.
    Interrupted,
    /// Any error not covered by the above variants.
    Other,
}

impl ErrorKind {
    fn as_str(self) -> &'static str {
        match self {
            ErrorKind::NotFound => "entity not found",
            ErrorKind::InvalidData => "invalid data",
            ErrorKind::InvalidInput => "invalid input parameter",
            ErrorKind::WriteZero => "write zero",
            ErrorKind::UnexpectedEof => "unexpected end of file",
            ErrorKind::Interrupted => "operation interrupted",
            ErrorKind::Other => "other error",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Internal representation
// ---------------------------------------------------------------------------

#[cfg(not(feature = "alloc"))]
enum Repr {
    Simple(ErrorKind),
    Custom {
        kind: ErrorKind,
        message: &'static str,
    },
}

#[cfg(feature = "alloc")]
enum Repr {
    Simple(ErrorKind),
    Custom {
        kind: ErrorKind,
        error: alloc::boxed::Box<dyn core::error::Error + Send + Sync>,
    },
}

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// The error type for I/O operations.
///
/// In `no_std` mode this is a lightweight wrapper around [`ErrorKind`] with
/// optional context. When the `alloc` feature is enabled the context may be
/// any type implementing [`core::error::Error`] + [`Send`] + [`Sync`].
pub struct Error {
    repr: Repr,
}

impl Error {
    /// Creates a new I/O error from an [`ErrorKind`] and a static message.
    ///
    /// This constructor is available with or without `alloc`.
    pub fn new_static(kind: ErrorKind, message: &'static str) -> Self {
        Error {
            repr: Repr::Custom {
                kind,
                #[cfg(not(feature = "alloc"))]
                message,
                #[cfg(feature = "alloc")]
                error: message.into(),
            },
        }
    }

    /// Creates a new I/O error from an [`ErrorKind`] and an error source.
    ///
    /// The source may be any `E: Into<Box<dyn Error + Send + Sync>>`.
    /// This matches the signature of [`std::io::Error::new`].
    #[cfg(feature = "alloc")]
    pub fn new<E>(kind: ErrorKind, error: E) -> Self
    where
        E: Into<alloc::boxed::Box<dyn core::error::Error + Send + Sync>>,
    {
        Error {
            repr: Repr::Custom {
                kind,
                error: error.into(),
            },
        }
    }

    /// Returns the corresponding [`ErrorKind`] for this error.
    pub fn kind(&self) -> ErrorKind {
        match &self.repr {
            Repr::Simple(kind) => *kind,
            Repr::Custom { kind, .. } => *kind,
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Simple(kind) => f.debug_tuple("Error").field(kind).finish(),
            #[cfg(not(feature = "alloc"))]
            Repr::Custom { kind, message } => f
                .debug_struct("Error")
                .field("kind", kind)
                .field("message", message)
                .finish(),
            #[cfg(feature = "alloc")]
            Repr::Custom { kind, error } => f
                .debug_struct("Error")
                .field("kind", kind)
                .field("error", error)
                .finish(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Simple(kind) => kind.fmt(f),
            #[cfg(not(feature = "alloc"))]
            Repr::Custom { message, .. } => f.write_str(message),
            #[cfg(feature = "alloc")]
            Repr::Custom { error, .. } => error.fmt(f),
        }
    }
}

impl core::error::Error for Error {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match &self.repr {
            Repr::Simple(_) => None,
            #[cfg(not(feature = "alloc"))]
            Repr::Custom { .. } => None,
            #[cfg(feature = "alloc")]
            Repr::Custom { error, .. } => error.source(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "alloc")]
    fn error_kind_display() {
        use alloc::string::ToString;
        assert_eq!(ErrorKind::NotFound.to_string(), "entity not found");
        assert_eq!(ErrorKind::InvalidData.to_string(), "invalid data");
        assert_eq!(
            ErrorKind::InvalidInput.to_string(),
            "invalid input parameter"
        );
        assert_eq!(ErrorKind::WriteZero.to_string(), "write zero");
        assert_eq!(
            ErrorKind::UnexpectedEof.to_string(),
            "unexpected end of file"
        );
        assert_eq!(ErrorKind::Interrupted.to_string(), "operation interrupted");
        assert_eq!(ErrorKind::Other.to_string(), "other error");
    }

    #[test]
    fn simple_error_preserves_kind() {
        let err = Error::from(ErrorKind::InvalidData);
        assert_eq!(err.kind(), ErrorKind::InvalidData);
    }

    #[test]
    fn static_error_preserves_kind() {
        let err = Error::new_static(ErrorKind::NotFound, "widget missing");
        assert_eq!(err.kind(), ErrorKind::NotFound);
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn custom_error_preserves_kind_and_message() {
        let err = Error::new(ErrorKind::NotFound, "widget missing");
        assert_eq!(err.kind(), ErrorKind::NotFound);
        let display = alloc::format!("{}", err);
        assert!(
            display.contains("widget missing"),
            "unexpected display: {display}"
        );
    }

    #[test]
    fn error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
