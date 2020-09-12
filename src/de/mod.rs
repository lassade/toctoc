//! Deserialization traits.
//!
//! Deserialization in toctoc works by returning a "place" into which data
//! may be written through the methods of the `Visitor` trait object.
//!
//! Use the `make_place!` macro to acquire a "place" type. A library may use a
//! single place type across all of its Deserialize impls, or each impl or each
//! module may use a private place type. There is no difference.
//!
//! A place is simply:
//!
//! ```rust
//! struct Place<T> {
//!     out: Option<T>,
//! }
//! ```
//!
//! Upon successful deserialization the output object is written as `Some(T)`
//! into the `out` field of the place.
//!
//! ## Deserializing a primitive
//!
//! The Visitor trait has a method corresponding to each supported primitive
//! type.
//!
//! ```rust
//! use toctoc::{make_place, Result};
//! use toctoc::de::{Deserialize, Visitor};
//!
//! make_place!(Place);
//!
//! struct MyBoolean(bool);
//!
//! // The Visitor trait has a selection of methods corresponding to different
//! // data types. We override the ones that our Rust type supports
//! // deserializing from, and write the result into the `out` field of our
//! // output place.
//! //
//! // These methods may perform validation and decide to return an error.
//! impl<'de> Visitor<'de> for Place<MyBoolean> {
//!     fn boolean(&mut self, b: bool) -> Result<()> {
//!         self.out = Some(MyBoolean(b));
//!         Ok(())
//!     }
//! }
//!
//! impl<'de> Deserialize<'de> for MyBoolean {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//!         // All Deserialize impls will look exactly like this. There is no
//!         // other correct implementation of Deserialize.
//!         Place::new(out)
//!     }
//! }
//! ```
//!
//! ## Deserializing a sequence
//!
//! In the case of a sequence (JSON array), the visitor method returns a builder
//! that can hand out places to write sequence elements one element at a time.
//!
//! ```rust
//! use toctoc::{make_place, Result};
//! use toctoc::de::{Deserialize, Seq, Visitor, Context};
//! use std::mem;
//!
//! make_place!(Place);
//!
//! struct MyVec<T>(Vec<T>);
//!
//! impl<'de, T: Deserialize<'de>> Visitor<'de> for Place<MyVec<T>> {
//!         fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
//!             let mut vec = vec![];
//!             let mut element = None;
//!             while s.visit(Deserialize::begin(&mut element), c)? {
//!                 element.take().map(|e| vec.push(e));
//!             }
//!             self.out = Some(MyVec(vec));
//!             Ok(())
//!         }
//! }
//!
//! impl<'de, T: Deserialize<'de>> Deserialize<'de> for MyVec<T> {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//!         // As mentioned, all Deserialize impls will look like this.
//!         Place::new(out)
//!     }
//! }
//! ```
//!
//! ## Deserializing a map or struct
//!
//! This code demonstrates what is generated for structs by
//! `#[derive(Deserialize)]`.
//!
//! ```rust
//! use toctoc::{make_place, Result, Error};
//! use toctoc::de::{Deserialize, Map, Visitor, Context};
//!
//! make_place!(Place);
//!
//! // The struct that we would like to deserialize.
//! struct Demo {
//!     code: u32,
//!     message: String,
//! }
//!
//! impl<'de> Visitor<'de> for Place<Demo> {
//!     fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
//!         let mut code = Deserialize::default();
//!         let mut message = Deserialize::default();
//!         while let Some(k) = m.next()? {
//!             match k {
//!                 "code" => m.visit(Deserialize::begin(&mut code), c)?,
//!                 "message" => m.visit(Deserialize::begin(&mut message), c)?,
//!                 _ => m.visit(Visitor::ignore(), c)?,
//!             }
//!         }
//!         let code = code.ok_or(Error::missing_field("code"))?;
//!         let message = message.unwrap_or_else(Default::default); // Optional field
//!         self.out = Some(Demo { code, message });
//!         Ok(())
//!     }
//! }
//!
//! impl<'de> Deserialize<'de> for Demo {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//!         // All Deserialize impls look like this.
//!         Place::new(out)
//!     }
//! }
//! ```

mod impls;

use crate::error::{Error, Result};
use crate::export::{Asset, Entity, Hint};

/// Trait for data structures that can be deserialized from a JSON string.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Deserialize<'de>: Sized {
    /// The only correct implementation of this method is:
    ///
    /// ```rust
    /// # use toctoc::make_place;
    /// # use toctoc::de::{Deserialize, Visitor};
    /// #
    /// # make_place!(Place);
    /// # struct S;
    /// # impl<'de> Visitor<'de> for Place<S> {}
    /// #
    /// # impl<'de> Deserialize<'de> for S {
    /// fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
    ///     Place::new(out)
    /// }
    /// # }
    /// ```
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de>;

    // Not public API. This method is only intended for Option<T>, should not
    // need to be implemented outside of this crate.
    #[doc(hidden)]
    #[inline]
    fn default() -> Option<Self> {
        None
    }
}

pub struct Deserializer<'a, 'de>(&'a mut dyn DeserializerTrait<'de>);

impl<'a, 'de> Deserializer<'a, 'de> {
    pub fn deserialize(
        self,
        value: &mut dyn Visitor<'de>,
        context: &mut dyn Context,
    ) -> Result<()> {
        self.0.deserialize(value, context)
    }
}

impl<'a, 'de, D: DeserializerTrait<'de>> From<&'a mut D> for Deserializer<'a, 'de> {
    fn from(d: &'a mut D) -> Self {
        Self(d)
    }
}

/// A data format that can deserialize any data structure supported by Toctoc.
pub trait DeserializerTrait<'de> {
    fn deserialize(
        &mut self,
        value: &mut dyn Visitor<'de>,
        context: &mut dyn Context,
    ) -> Result<()>;
}

// * MOD Added some optional context to certain types of objects
/// Trait that can write data into an output place.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Visitor<'de> {
    fn null(&mut self, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        Err(Error::not_expected("null"))?
    }

    fn boolean(&mut self, b: bool) -> Result<()> {
        let _ = b;
        Err(Error::not_expected("boolean"))?
    }

    fn string(&mut self, s: &'de str, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = s;
        Err(Error::not_expected("string"))?
    }

    fn negative(&mut self, n: i64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error::not_expected("negative"))?
    }

    fn nonnegative(&mut self, n: u64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error::not_expected("nonnegative"))?
    }

    fn double(&mut self, n: f64) -> Result<()> {
        let _ = n;
        Err(Error::not_expected("double"))?
    }

    fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
        let _ = s;
        let _ = c;
        Err(Error::not_expected("seq"))?
    }

    fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
        let _ = m;
        let _ = c;
        Err(Error::not_expected("map"))?
    }

    // * MOD: Extra deserialization functions
    fn single(&mut self, n: f32) -> Result<()> {
        let _ = n;
        Err(Error::not_expected("single"))?
    }

    fn bytes(&mut self, b: &'de [u8], c: &mut dyn Context) -> Result<()> {
        let _ = b;
        let _ = c;
        Err(Error::not_expected("bytes"))?
    }
}

pub trait Seq<'de> {
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<bool>;
}

pub trait Map<'de> {
    fn next(&mut self) -> Result<Option<&'de str>>;
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()>;
}

/// Trait that can resolves complex types based on some context.
#[cfg(not(feature = "any-context"))]
pub trait Context {
    fn entity(&mut self, e: Hint) -> Result<Entity> {
        let _ = e;
        Err(Error::not_expected("entity"))?
    }

    fn asset(&mut self, a: Hint) -> Result<Asset> {
        let _ = a;
        Err(Error::not_expected("asset"))?
    }
}

#[cfg(not(feature = "any-context"))]
impl Context for () {}

#[cfg(feature = "any-context")]
pub trait Context = std::any::Any;
