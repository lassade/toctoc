//! Deserialization traits.
//!
//! Deserialization in knocknoc works by returning a "place" into which data
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
//! use knocknoc::{make_place, Result};
//! use knocknoc::de::{Deserialize, Visitor};
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
//! use knocknoc::{make_place, Result};
//! use knocknoc::de::{Deserialize, Seq, Visitor, Context};
//! use std::mem;
//!
//! make_place!(Place);
//!
//! struct MyVec<T>(Vec<T>);
//!
//! impl<'de, T: Deserialize<'de>> Visitor<'de> for Place<MyVec<T>> {
//!     fn seq<'a>(&'a mut self) -> Result<Box<dyn Seq<'de> + 'a>>
//!     where
//!         'de: 'a
//!     {
//!         Ok(Box::new(VecBuilder {
//!             out: &mut self.out,
//!             vec: Vec::new(),
//!             element: None,
//!         }))
//!     }
//! }
//!
//! struct VecBuilder<'a, T: 'a> {
//!     // At the end, output will be written here.
//!     out: &'a mut Option<MyVec<T>>,
//!     // Previous elements are accumulated here.
//!     vec: Vec<T>,
//!     // Next element will be placed here.
//!     element: Option<T>,
//! }
//!
//! impl<'a, 'de: 'a, T: Deserialize<'de>> Seq<'de> for VecBuilder<'a, T> {
//!     fn element(&mut self) -> Result<&mut dyn Visitor<'de>> {
//!         // Free up the place by transfering the most recent element
//!         // into self.vec.
//!         self.vec.extend(self.element.take());
//!         // Hand out a place to write the next element.
//!         Ok(Deserialize::begin(&mut self.element))
//!     }
//!
//!     fn finish(&mut self, _: &mut dyn Context) -> Result<()> {
//!         // Transfer the last element.
//!         self.vec.extend(self.element.take());
//!         // Move the output object into self.out.
//!         let vec = mem::replace(&mut self.vec, Vec::new());
//!         *self.out = Some(MyVec(vec));
//!         Ok(())
//!     }
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
//! use knocknoc::{make_place, Result};
//! use knocknoc::de::{Deserialize, Map, Visitor, Context};
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
//!     fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'de> + 'a>>
//!     where
//!         'de: 'a
//!     {
//!         // Like for sequences, we produce a builder that can hand out places
//!         // to write one struct field at a time.
//!         Ok(Box::new(DemoBuilder {
//!             code: None,
//!             message: None,
//!             out: &mut self.out,
//!         }))
//!     }
//! }
//!
//! struct DemoBuilder<'a> {
//!     code: Option<u32>,
//!     message: Option<String>,
//!     out: &'a mut Option<Demo>,
//! }
//!
//! impl<'a, 'de: 'a> Map<'de> for DemoBuilder<'a> {
//!     fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'de>> {
//!         // Figure out which field is being deserialized and return a place
//!         // to write it.
//!         //
//!         // The code here ignores unrecognized fields but an implementation
//!         // would be free to return an error instead. Similarly an
//!         // implementation may want to check for duplicate fields by
//!         // returning an error if the current field already has a value.
//!         match k {
//!             "code" => Ok(Deserialize::begin(&mut self.code)),
//!             "message" => Ok(Deserialize::begin(&mut self.message)),
//!             _ => Ok(Visitor::ignore()),
//!         }
//!     }
//!
//!     fn finish(&mut self, _: &mut dyn Context) -> Result<()> {
//!         // Make sure we have every field and then write the output object
//!         // into self.out.
//!         let code = self.code.take().ok_or(knocknoc::Error)?;
//!         let message = self.message.take().ok_or(knocknoc::Error)?;
//!         *self.out = Some(Demo { code, message });
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
use crate::export::{Asset, Entity};

/// Trait for data structures that can be deserialized from a JSON string.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Deserialize<'de>: Sized {
    /// The only correct implementation of this method is:
    ///
    /// ```rust
    /// # use knocknoc::make_place;
    /// # use knocknoc::de::{Deserialize, Visitor};
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

// * MOD Added some optional context to certain types of objects
/// Trait that can write data into an output place.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Visitor<'de> {
    fn null(&mut self, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        Err(Error)?
    }

    fn boolean(&mut self, b: bool) -> Result<()> {
        let _ = b;
        Err(Error)?
    }

    fn string(&mut self, s: &'de str, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = s;
        Err(Error)?
    }

    fn negative(&mut self, n: i64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error)?
    }

    fn nonnegative(&mut self, n: u64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error)?
    }

    fn double(&mut self, n: f64) -> Result<()> {
        let _ = n;
        Err(Error)?
    }

    fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
        let _ = s;
        let _ = c;
        Err(Error)?
    }

    fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
        let _ = m;
        let _ = c;
        Err(Error)?
    }

    // * MOD: Extra deserialization functions
    fn single(&mut self, n: f32) -> Result<()> {
        let _ = n;
        Err(Error)?
    }

    fn bytes(&mut self, b: &'de [u8], c: &mut dyn Context) -> Result<()> {
        let _ = b;
        let _ = c;
        Err(Error)?
    }
}

pub trait Seq<'de> {
    //fn next(&mut self) -> Result<bool>;
    //fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()>;
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<bool>;
}

pub trait Map<'de> {
    fn next(&mut self) -> Result<Option<&'de str>>;
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()>;
}

pub enum Hint<'a> {
    Null,
    Number(u64),
    Str(&'a str),
    Bytes(&'a [u8]),
}

/// Trait that can resolves complex types based on some context.
pub trait Context {
    fn entity(&mut self, e: Hint) -> Result<Entity> {
        let _ = e;
        Err(Error)?
    }

    fn asset(&mut self, a: Hint) -> Result<Asset> {
        let _ = a;
        Err(Error)?
    }
}

impl Context for () {}
