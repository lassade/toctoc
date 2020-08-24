use std::fmt::{self, Display};

/// Error type when deserialization fails.
///
/// knocknoc errors contain no information about what went wrong. **If you need
/// more than no information, use Serde.**
#[derive(Copy, Clone, Debug)]
pub struct Error;

/// Result type returned by deserialization functions.
#[cfg(not(feature = "error"))]
pub type Result<T> = std::result::Result<T, Error>;

/// Result type returned by deserialization functions.
#[cfg(feature = "error")]
pub type Result<T> = anyhow::Result<T>;

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("knocknoc error")
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "knocknoc error"
    }
}
