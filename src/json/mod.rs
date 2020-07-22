//! JSON data format.
//!
//! [See the crate level doc](../index.html#example) for an example of
//! serializing and deserializing JSON.

mod ser;
pub use self::ser::to_string;

#[cfg(not(feature = "simd"))]
mod de;
#[cfg(not(feature = "simd"))]
pub use self::de::from_str;

mod value;
pub use self::value::Value;

mod number;
pub use self::number::Number;

mod array;
pub use self::array::Array;

mod object;
pub use self::object::Object;

mod drop;

#[cfg(feature = "simd")]
mod simd;
#[cfg(feature = "simd")]
pub use simd::from_str;

/// Any string ending with `\u0010` ascii control char,
/// should be treated as hex encoded binary data
pub const HEX_HINT: char = '\x10';

/// Utf8 escaped string for `HEX_HINT` char.
pub const HEX_HINT_ESCAPED: &'static str = r#"\u0010"#;