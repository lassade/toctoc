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
//! impl Visitor for Place<MyBoolean> {
//!     fn boolean(&mut self, b: bool) -> Result<()> {
//!         self.out = Some(MyBoolean(b));
//!         Ok(())
//!     }
//! }
//!
//! impl Deserialize for MyBoolean {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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
//! impl<T: Deserialize> Visitor for Place<MyVec<T>> {
//!     fn seq(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Seq + '_>> {
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
//! impl<'a, T: Deserialize> Seq for VecBuilder<'a, T> {
//!     fn element(&mut self) -> Result<&mut dyn Visitor> {
//!         // Free up the place by transfering the most recent element
//!         // into self.vec.
//!         self.vec.extend(self.element.take());
//!         // Hand out a place to write the next element.
//!         Ok(Deserialize::begin(&mut self.element))
//!     }
//!
//!     fn finish(&mut self) -> Result<()> {
//!         // Transfer the last element.
//!         self.vec.extend(self.element.take());
//!         // Move the output object into self.out.
//!         let vec = mem::replace(&mut self.vec, Vec::new());
//!         *self.out = Some(MyVec(vec));
//!         Ok(())
//!     }
//! }
//!
//! impl<T: Deserialize> Deserialize for MyVec<T> {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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
//! impl Visitor for Place<Demo> {
//!     fn map(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Map + '_>> {
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
//! impl<'a> Map for DemoBuilder<'a> {
//!     fn key(&mut self, k: &str) -> Result<&mut dyn Visitor> {
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
//!     fn finish(&mut self) -> Result<()> {
//!         // Make sure we have every field and then write the output object
//!         // into self.out.
//!         let code = self.code.take().ok_or(knocknoc::Error)?;
//!         let message = self.message.take().ok_or(knocknoc::Error)?;
//!         *self.out = Some(Demo { code, message });
//!         Ok(())
//!     }
//! }
//!
//! impl Deserialize for Demo {
//!     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
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
pub trait Deserialize<'i> : Sized {
    /// The only correct implementation of this method is:
    ///
    /// ```rust
    /// # use knocknoc::make_place;
    /// # use knocknoc::de::{Deserialize, Visitor};
    /// #
    /// # make_place!(Place);
    /// # struct S;
    /// # impl Visitor for Place<S> {}
    /// #
    /// # impl Deserialize for S {
    /// fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
    ///     Place::new(out)
    /// }
    /// # }
    /// ```
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i>;

    // Not public API. This method is only intended for Option<T>, should not
    // need to be implemented outside of this crate.
    #[doc(hidden)]
    #[inline]
    fn default() -> Option<Self> {
        None
    }
}

// * MOD Added some optional contex to certain types of objects
/// Trait that can write data into an output place.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Visitor<'i> {
    fn null(&mut self, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        Err(Error)
    }

    fn boolean(&mut self, b: bool) -> Result<()> {
        let _ = b;
        Err(Error)
    }

    fn string(&mut self, s: &'i str, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = s;
        Err(Error)
    }

    fn negative(&mut self, n: i64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error)
    }

    fn nonnegative(&mut self, n: u64, c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = n;
        Err(Error)
    }

    fn double(&mut self, n: f64) -> Result<()> {
        let _ = n;
        Err(Error)
    }

    fn seq<'a>(&'a mut self) -> Result<Box<dyn Seq<'i> + 'a>> 
    where
        'i: 'a
    {
        Err(Error)
    }
    
    fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'i> + 'a>>
    where
        'i: 'a
    {
        Err(Error)
    }

    // * MOD: Extra deserializaiton functions
    fn single(&mut self, n: f32) -> Result<()> {
        let _ = n;
        Err(Error)
    }

    fn bytes(&mut self, b: &'i [u8], c: &mut dyn Context) -> Result<()> {
        let _ = c;
        let _ = b;
        Err(Error)
    }
}

/// Trait that can hand out places to write sequence elements.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Seq<'i> {
    fn element(&mut self) -> Result<&mut dyn Visitor<'i>>;
    fn finish(&mut self, c: &mut dyn Context) -> Result<()>;
}

/// Trait that can hand out places to write values of a map.
///
/// [Refer to the module documentation for examples.][::de]
pub trait Map<'i> {
    fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'i>>;
    fn finish(&mut self, c: &mut dyn Context) -> Result<()>;
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
        Err(Error)
    }

    fn asset(&mut self, a: Hint) -> Result<Asset> {
        let _ = a;
        Err(Error)
    }
}