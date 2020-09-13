//! # Struct
//!
//! ```rust
//! use toctoc::{json, Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! struct Example<'a> {
//!     code: u32,
//!     #[toctoc(rename = msg)]
//!     message: &'a str,
//!     #[toctoc(skip)]
//!     ignore: (),
//! }
//!
//! fn main() -> toctoc::Result<()> {
//!     let example = Example {
//!         code: 200,
//!         message: "reminiscent of Serde",
//!         ignore: (),
//!     };
//!
//!     let mut j = json::to_string(&example, &mut ());
//!     println!("{}", j);
//!
//!     let out: Example = json::from_str(&mut j, &mut ())?;
//!     println!("{:?}", out);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Enum
//!
//! ```rust
//! use toctoc::{json, Serialize, Deserialize};
//!
//! #[derive(Debug, PartialEq, Serialize)]
//! enum WXYZ {
//!     W { a: i32, b: i32 },
//!     X(i32, i32),
//!     Y(i32),
//!     Z,
//! }
//! ```

#![cfg_attr(feature = "any-context", feature(trait_alias))]
#![doc(html_root_url = "https://docs.rs/toctoc/0.1.13")]
#![allow(
    clippy::needless_doctest_main,
    // Regression causing false positives:
    // https://github.com/rust-lang/rust-clippy/issues/5343
    clippy::useless_transmute,
    // Clippy bug: https://github.com/rust-lang/rust-clippy/issues/5704
    clippy::unnested_or_patterns,
)]

#[doc(hidden)]
pub use toctoc_internal::*;

// These derives were renamed from MiniTrait -> Trait with the release of Rust
// 1.30.0. Keep exposing the old names for backward compatibility but remove in
// the next major version of toctoc.
#[doc(hidden)]
pub use toctoc_internal::{Deserialize as MiniDeserialize, Serialize as MiniSerialize};

// Not public API.
#[doc(hidden)]
pub mod export;

#[macro_use]
mod careful;

#[macro_use]
mod place;

#[macro_use]
mod error;
pub mod buffer;
pub mod bytes;
mod ignore;
mod owned;

pub mod bson;
pub mod de;
pub mod json;
pub mod ser;

#[doc(inline)]
pub use crate::de::Deserialize;
#[doc(inline)]
pub use crate::de::Deserializer;
pub use crate::error::{Error, Result};
#[doc(inline)]
pub use crate::ser::Serialize;
#[doc(inline)]
pub use crate::ser::Serializer;

make_place!(Place);

#[cfg(target_endian = "big")]
#[allow(unused)]
pub fn check_endianness() {
    compile_error!("`Bytes` are always assumed to be in little endian, this will break json hexadecimal strings;\
        bson strings and binary data doesn't support big endian");
}
