//! JSON data format.
//!
//! [See the crate level doc](../index.html#example) for an example of
//! serializing and deserializing JSON.

mod ser;
pub use self::ser::to_string;

mod de;
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

/// String pos-fixed with `\u{0010}` control char, should be treated as
/// hex encoded binary data
pub const HEX_HINT: char = '\u{10}';

/// Utf8 escaped string for `HEX_HINT` char.
pub const HEX_HINT_ESCAPED: &'static str = r#"\u0010"#;