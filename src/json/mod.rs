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

mod owned;
pub use self::owned::{from_str_owned, Owned};
