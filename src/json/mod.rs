//! JSON data format.
//!
//! [See the crate level doc](../index.html#example) for an example of
//! serializing and deserializing JSON.

mod ser;
pub use self::ser::to_string;
pub use ser::JsonSer;

pub use export::*;

#[cfg(not(feature = "simd"))]
mod de;

#[cfg(not(feature = "simd"))]
mod export {
    pub use super::de::from_str;
}

#[cfg(feature = "simd")]
mod simd;

#[cfg(feature = "simd")]
mod export {
    pub use super::simd::from_str;
    pub use super::simd::JsonDe;
}

mod value;
pub use self::value::Value;

mod number;
pub use self::number::Number;

mod array;
pub use self::array::Array;

mod object;
pub use self::object::Object;

mod drop;

mod owned;
pub use self::owned::{from_str_owned, Owned};
