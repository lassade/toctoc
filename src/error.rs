use std::fmt::{self, Debug, Display};

#[cfg(feature = "ufmt1")]
#[macro_use]
macro_rules! err {
    // IMPORTANT use `tt` fragments instead of `expr` fragments (i.e. `$($exprs:expr),*`)
    ($($tt:tt)*) => {{
        let mut s = String::new();
        ufmt::uwrite!(&mut s, $($tt)*).unwrap();
        Error(s)
    }}
}

#[cfg(not(feature = "ufmt1"))]
#[macro_use]
macro_rules! err {
    // IMPORTANT use `tt` fragments instead of `expr` fragments (i.e. `$($exprs:expr),*`)
    ($($tt:tt)*) => {{
        Error(format!($($tt)*))
    }}
}

///////////////////////////////////////////////////////////////////////////////

#[cfg(not(feature = "error"))]
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "error")]
pub type Result<T> = anyhow::Result<T>;

///////////////////////////////////////////////////////////////////////////////

/// Error type when deserialization fails.
///
/// Kept just like a `String` because it's simpler. In the current target application
/// the error will be always logged so the used can take action. Therefore there is no reason
/// to have a underlying enum representation just to be later converted into a string any way.
pub struct Error(pub(crate) String);

impl Error {
    pub(crate) fn append_line_and_column(mut self, line: usize, column: usize) -> Self {
        if cfg!(feature = "ufmt1") {
            ufmt::uwrite!(&mut self.0, ", {}:{}", line, column).unwrap();
        } else {
            use std::fmt::Write;
            write!(&mut self.0, ", {}:{}", line, column).unwrap();
        }
        self
    }

    pub fn unknown() -> Self {
        Self("unknown error".to_string())
    }

    /// Generic error
    pub fn generic(err: String) -> Self {
        Self(err)
    }

    /// Not expected visit
    pub fn not_expected(msg: &str) -> Self {
        err!("not expected `{}`", msg)
    }

    /// Was expecting something
    pub fn expecting(msg: &str) -> Self {
        err!("expecting `{}`", msg)
    }

    /// Missing field inside a map
    pub fn missing_field(field: &str) -> Self {
        err!("missing field `{}`", field)
    }

    /// Missing element inside a sequence, most likely a tuple
    pub fn missing_element(index: usize) -> Self {
        err!("missing tuple element {}", index)
    }

    pub fn unknown_variant(variant: &str) -> Self {
        err!("unknown variant `{}`", variant)
    }

    // /// Invalid char
    // pub fn invalid_char(ch: char) -> Self {
    //     let unicode = (ch as u32).to_le_bytes();
    //     // TODO: Use encode_noalloc when available to avoid allocations
    //     let hex = bintext::hex::encode(&unicode[..]);
    //     err!("invalid char `{}` (\\u{})", ch, hex)
    // }

    /// Found an invalid UTF8 sequence
    pub fn invalid_utf8() -> Self {
        Self("invalid utf8".to_string())
    }

    /// When there isn't enough alignment needed to decode and align a hex sequence
    pub fn not_enough_offset(needed: usize, current: usize) -> Self {
        err!(
            "not enough offset to align hex string, needed {} but got {}",
            needed,
            current
        )
    }

    /// Document specifies the need for a higher rank alignment
    /// than it actually got, so the alignment of the underlying data
    /// can't be guaranteed
    pub fn lower_alignment_rank(needed: usize, current: usize) -> Self {
        err!(
            "lower than expected alignment rank, needed: {} got: {}",
            needed,
            current
        )
    }

    /// Data isn't aligned;
    ///
    /// May happen when a BSON containing aligned binary data isn't properly
    /// aligned
    pub fn not_aligned(align: usize, offset: usize) -> Self {
        err!(
            "bytes not properly aligned, align {} is off by {}",
            align,
            offset
        )
    }

    /// Invalid map key, probably mean that a key couldn't be created
    /// using the `FromStr` trait using this input
    pub fn invalid_map_key(mut key: String) -> Self {
        key.insert_str(0, "invalid map key ");
        Self(key)
    }

    /// Out of range of some type
    pub fn out_of_range(ty: &str) -> Self {
        err!("out of range of `{}`", ty)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}
