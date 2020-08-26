use std::fmt::{self, Display};

/// Unknown error alias
#[allow(non_upper_case_globals)]
pub const Error: Error1 = Error1::Unknown;

/// Result type returned by deserialization functions.
#[cfg(not(feature = "error"))]
pub type Result<T> = std::result::Result<T, Error1>;

/// Result type returned by deserialization functions.
#[cfg(feature = "error")]
pub type Result<T> = anyhow::Result<T>;

///////////////////////////////////////////////////////////////////////////////

/// Error type when deserialization fails.
#[derive(Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Error1 {
    Unknown,
    /// Not expected visit
    NotExpected(&'static str),
    /// Missing field inside a map
    MissingField(&'static str),
    /// Missing element inside a sequence, most likely a tuple
    MissingElement(usize),
    /// Invalid char at (text)
    InvalidCharAt {
        ch: u8,
        line: usize,
        column: usize,
    },
    /// Invalid byte at (binary)
    InvalidByte {
        byte: u8,
        index: usize,
    },
    /// Found an invalid UTF8 sequence
    InvalidUtf8 {
        line: usize,
        column: usize,
    },
    /// Found an invalid UTF8 sequence at some index (used for binary formats)
    InvalidUtf8AtIndex {
        index: usize,
    },
    /// When there isn't enough alignment needed to decode and align a hex sequence
    NotEnoughOffset {
        line: usize,
        column: usize,
        needed: u8,
        current: u8,
    },
    /// Document specifies the need for a higher rank alignment
    /// than it actually got, so the alignment of the underlying data
    /// can't be guaranteed
    LowerAlignmentRank {
        index: usize,
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

impl Display for Error1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error1::Unknown => f.write_str("unknown error"),
            Error1::NotExpected(name) => write!(f, "not expected `{}`", name),
            Error1::MissingField(field) => write!(f, "missing field `{}`", field),
            Error1::MissingElement(index) => write!(f, "missing tuple element {}", index),
            Error1::InvalidCharAt { ch, line, column } => {
                write!(f, "invalid char `{}` at {}:{}", *ch as char, line, column)
            }
            Error1::InvalidByte { byte, index } => {
                write!(f, "invalid byte `\\x{:02x}` at {}", byte, index)
            }
            Error1::InvalidUtf8 { line, column } => write!(f, "invalid utf8 {}:{}", line, column),
            Error1::InvalidUtf8AtIndex { index } => write!(f, "invalid utf8 at index {}", index),
            Error1::NotEnoughOffset {
                line,
                column,
                needed,
                current,
            } => write!(
                f,
                "not enough offset to align hex string, needed {} but got {} at {}:{}",
                needed, current, line, column
            ),
            Error1::LowerAlignmentRank {
                index,
                needed,
                current,
            } => write!(
                f,
                "lower than expected alignment rank, needed: {} got: {} at index {}",
                needed, current, index
            ),
            Error1::NotAligned { align, offset } => write!(
                f,
                "bytes not properly aligned, align {} is off by {}",
                align, offset
            ),
        }
    }
}

impl std::error::Error for Error1 {}
