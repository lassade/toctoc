use std::fmt::{self, Display};

#[cfg(not(feature = "error"))]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "error")]
pub type Result<T> = anyhow::Result<T>;

///////////////////////////////////////////////////////////////////////////////

/// Error type when deserialization fails.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Unknown,
    /// Generic error
    Generic(Box<dyn std::error::Error>),
    /// Not expected visit
    NotExpected(&'static str),
    /// Was expecting something
    Expecting(&'static str),
    /// Missing field inside a map
    MissingField(&'static str),
    /// Missing element inside a sequence, most likely a tuple
    MissingElement(usize),
    /// Invalid char
    InvalidChar(char),
    /// Invalid map key, probably mean that a key couldn't be created
    /// using the `FromStr` trait using this input
    InvalidMapKey(String),
    /// Out of range of some type
    OutOfRange(&'static str),
    /// Found an invalid UTF8 sequence
    InvalidUtf8,
    /// When there isn't enough alignment needed to decode and align a hex sequence
    NotEnoughOffset {
        needed: u8,
        current: u8,
    },
    /// Document specifies the need for a higher rank alignment
    /// than it actually got, so the alignment of the underlying data
    /// can't be guaranteed
    LowerAlignmentRank {
        needed: u8,
        current: u8,
    },
    /// Data isn't aligned;
    ///
    /// May happen when a BSON containing aligned binary data isn't properly
    /// aligned
    NotAligned {
        align: u8,
        offset: u8,
    },
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Unknown => f.write_str("unknown error"),
            Error::Generic(err) => err.fmt(f),
            Error::NotExpected(msg) => write!(f, "not expected `{}`", msg),
            Error::Expecting(msg) => write!(f, "expecting `{}`", msg),
            Error::MissingField(field) => write!(f, "missing field `{}`", field),
            Error::MissingElement(index) => write!(f, "missing tuple element {}", index),
            Error::InvalidChar(ch) => write!(f, "invalid char `{}` (\\x{:02x})", *ch, *ch as u8),
            Error::InvalidUtf8 => write!(f, "invalid utf8"),
            Error::NotEnoughOffset { needed, current } => write!(
                f,
                "not enough offset to align hex string, needed {} but got {}",
                needed, current
            ),
            Error::LowerAlignmentRank { needed, current } => write!(
                f,
                "lower than expected alignment rank, needed: {} got: {}",
                needed, current
            ),
            Error::NotAligned { align, offset } => write!(
                f,
                "bytes not properly aligned, align {} is off by {}",
                align, offset
            ),
            Error::InvalidMapKey(key) => write!(f, "invalid map key `{}`", key),
            Error::OutOfRange(ty) => write!(f, "out of range of `{}`", ty),
        }
    }
}

impl std::error::Error for Error {}

///////////////////////////////////////////////////////////////////////////////

/// Result type returned by deserialization functions.
#[cfg(not(feature = "error"))]
pub type ResultAt<T> = std::result::Result<T, ErrorAt>;

/// Result type returned by deserialization functions.
#[cfg(feature = "error")]
pub type ResultAt<T> = anyhow::Result<T>;

///////////////////////////////////////////////////////////////////////////////

/// Same as `Error` but also contains info about where in the buffer the error
/// has ocurred
#[derive(Debug)]
pub struct ErrorAt {
    pub line: usize,
    pub column: usize,
    pub err: Error,
}

impl ErrorAt {
    fn from_bare(line: usize, column: usize, err: Error) -> Self {
        ErrorAt { line, column, err }
    }
}

impl Display for ErrorAt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}:{}", self.err, self.line, self.column)
    }
}

impl std::error::Error for ErrorAt {}
