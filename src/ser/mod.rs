//! Serialization traits.
//!
//! Serialization in knocknoc works by traversing an input object and
//! decomposing it iteratively into a stream of fragments.
//!
//! ## Serializing a primitive
//!
//! ```rust
//! use knocknoc::ser::{Fragment, Serialize, Context};
//!
//! // The data structure that we want to serialize as a primitive.
//! struct MyBoolean(bool);
//!
//! impl Serialize for MyBoolean {
//!     fn begin(&self, _c: &dyn Context) -> Fragment {
//!         Fragment::Bool(self.0)
//!     }
//! }
//! ```
//!
//! ## Serializing a sequence
//!
//! ```rust
//! use knocknoc::ser::{Fragment, Seq, Serialize, Context};
//!
//! // Some custom sequence type that we want to serialize.
//! struct MyVec<T>(Vec<T>);
//!
//! impl<T: Serialize> Serialize for MyVec<T> {
//!     fn begin(&self, _c: &dyn Context) -> Fragment {
//!         Fragment::Seq(Box::new(SliceStream { iter: self.0.iter() }))
//!     }
//! }
//!
//! struct SliceStream<'a, T: 'a> {
//!     iter: std::slice::Iter<'a, T>,
//! }
//!
//! impl<'a, T: Serialize> Seq for SliceStream<'a, T> {
//!     fn next(&mut self) -> Option<&dyn Serialize> {
//!         let element = self.iter.next()?;
//!         Some(element)
//!     }
//! }
//! ```
//!
//! ## Serializing a map or struct
//!
//! This code demonstrates what is generated for structs by
//! `#[derive(Serialize)]`.
//!
//! ```rust
//! use knocknoc::ser::{Fragment, Map, Serialize, Context};
//! use std::borrow::Cow;
//!
//! // The struct that we would like to serialize.
//! struct Demo {
//!     code: u32,
//!     message: String,
//! }
//!
//! impl Serialize for Demo {
//!     fn begin(&self, _c: &dyn Context) -> Fragment {
//!         Fragment::Map(Box::new(DemoStream {
//!             data: self,
//!             state: 0,
//!         }))
//!     }
//! }
//!
//! struct DemoStream<'a> {
//!     data: &'a Demo,
//!     state: usize,
//! }
//!
//! impl<'a> Map for DemoStream<'a> {
//!     fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
//!         let state = self.state;
//!         self.state += 1;
//!         match state {
//!             0 => Some((Cow::Borrowed("code"), &self.data.code)),
//!             1 => Some((Cow::Borrowed("message"), &self.data.message)),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

mod impls;

use std::borrow::Cow;
use crate::export::{Asset, Entity};

/// One unit of output produced during serialization.
///
/// [Refer to the module documentation for examples.][::ser]
pub enum Fragment<'a> {
    Null,
    Bool(bool),
    Str(Cow<'a, str>),
    U64(u64),
    I64(i64),
    F64(f64),
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
    // * MOD: More types to better support binary formats
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    F32(f32),
    /// Binary data, should be serialized as hex string when binary
    /// output is not supported like in json
    Bin {
        /// Unaligned binary data
        bytes: Cow<'a, [u8]>,
        /// Bytes alignment, must be ensured
        align: usize,
    },
}

/// Trait for data structures that can be serialized to a JSON string.
///
/// [Refer to the module documentation for examples.][::ser]
pub trait Serialize {
    fn begin(&self, context: &dyn Context) -> Fragment;
}

/// Trait that can iterate elements of a sequence.
///
/// [Refer to the module documentation for examples.][::ser]
pub trait Seq {
    fn next(&mut self) -> Option<&dyn Serialize>;
}

/// Trait that can iterate key-value entries of a map or struct.
///
/// [Refer to the module documentation for examples.][::ser]
pub trait Map {
    fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)>;
}

/// Trait that can translate complex types based on some context
/// into serializable fragments
pub trait Context {
    fn entity(&self, e: Entity) -> Fragment {
        let _ = e;
        Fragment::Null
    }

    fn asset(&self, a: Asset) -> Fragment {
        let _ = a;
        Fragment::Null
    }
}