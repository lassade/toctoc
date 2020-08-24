//! Serialization traits.
//!
//! Serialization in knocknoc works by traversing an input object and
//! decomposing it iteratively into a stream of fragments.
//!
//! ## Serializing a primitive
//!
//! ```rust
//! use knocknoc::ser::{Serialize, Visitor, Context, Done};
//!
//! // The data structure that we want to serialize as a primitive.
//! struct MyBoolean(bool);
//!
//! impl Serialize for MyBoolean {
//!     fn begin(&self, v: Visitor, _: &dyn Context) -> Done {
//!         v.boolean(self.0)
//!     }
//! }
//! ```
//!
//! ## Serializing a sequence
//!
//! ```rust
//! use knocknoc::ser::{Serialize, Visitor, Context, Done};
//!
//! // Some custom sequence type that we want to serialize.
//! struct MyVec<T>(Vec<T>);
//!
//! impl<T: Serialize> Serialize for MyVec<T> {
//!     fn begin(&self, v: Visitor, context: &dyn Context) -> Done {
//!        let mut seq = v.seq();
//!        for e in &self.0 {
//!            seq = seq.element(e, context);
//!        }
//!        seq.done()
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
//! use knocknoc::ser::{Serialize, Visitor, Context, Done};
//!
//! // The struct that we would like to serialize.
//! struct Demo {
//!     code: u32,
//!     message: String,
//! }
//!
//! impl Serialize for Demo {
//!     fn begin(&self, v: Visitor, context: &dyn Context) -> Done {
//!         v.map()
//!             .field("code", &self.code, context)
//!             .field("message", &self.message, context)
//!             .done()
//!     }
//! }
//! ```

mod impls;

use crate::export::{Asset, Entity};

/// Trait for data structures that can be serialized to a JSON string.
///
/// [Refer to the module documentation for examples.][::ser]
pub trait Serialize {
    fn begin(&self, v: Visitor, context: &dyn Context) -> Done;
}

////////////////////////////////////////////////////////////////////////////////

pub struct Done(());

/// Safe interface to proper call `Ser` functions
pub struct Visitor<'a> {
    s: &'a mut dyn Serializer,
}

impl<'a, S: Serializer> From<&'a mut S> for Visitor<'a> {
    #[inline(always)]
    fn from(s: &'a mut S) -> Self {
        Visitor { s }
    }
}

impl<'a> Visitor<'a> {
    #[inline(always)]
    pub fn null(self) -> Done {
        self.s.null();
        Done(())
    }

    #[inline(always)]
    pub fn boolean(self, b: bool) -> Done {
        self.s.boolean(b);
        Done(())
    }

    #[inline(always)]
    pub fn string(self, s: &str) -> Done {
        self.s.string(s);
        Done(())
    }

    #[inline(always)]
    pub fn sbyte(self, n: i8) -> Done {
        self.s.sbyte(n);
        Done(())
    }

    #[inline(always)]
    pub fn int(self, n: i32) -> Done {
        self.s.int(n);
        Done(())
    }

    #[inline(always)]
    pub fn long(self, n: i64) -> Done {
        self.s.long(n);
        Done(())
    }

    #[inline(always)]
    pub fn byte(self, n: u8) -> Done {
        self.s.byte(n);
        Done(())
    }

    #[inline(always)]
    pub fn uint(self, n: u32) -> Done {
        self.s.uint(n);
        Done(())
    }

    #[inline(always)]
    pub fn ulong(self, n: u64) -> Done {
        self.s.ulong(n);
        Done(())
    }

    #[inline(always)]
    pub fn single(self, n: f32) -> Done {
        self.s.single(n);
        Done(())
    }

    #[inline(always)]
    pub fn double(self, n: f64) -> Done {
        self.s.double(n);
        Done(())
    }

    #[inline(always)]
    pub fn bytes(self, b: &[u8], align: usize) -> Done {
        self.s.bytes(b, align);
        Done(())
    }

    #[inline(always)]
    pub fn seq(self) -> Seq<'a> {
        Seq { s: self.s.seq() }
    }

    #[inline(always)]
    pub fn map(self) -> Map<'a> {
        Map { m: self.s.map() }
    }
}

/// Safe interface to proper call `SerializeSeq` functions
pub struct Seq<'a> {
    s: &'a mut dyn SerializerSeq,
}

impl<'a> Seq<'a> {
    #[inline(always)]
    pub fn element(self, s: &dyn Serialize, c: &dyn Context) -> Self {
        self.s.element(s, c);
        self
    }

    #[inline(always)]
    pub fn done(self) -> Done {
        self.s.done();
        Done(())
    }
}

/// Safe interface to proper call `SerializeSeq` functions
pub struct Map<'a> {
    m: &'a mut dyn SerializerMap,
}

impl<'a> Map<'a> {
    #[inline(always)]
    pub fn field(self, k: &str, s: &dyn Serialize, c: &dyn Context) -> Self {
        self.m.field(k, s, c);
        self
    }

    #[inline(always)]
    pub fn done(self) -> Done {
        self.m.done();
        Done(())
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Serializer {
    fn null(&mut self);

    fn boolean(&mut self, b: bool);

    fn string(&mut self, s: &str);

    #[inline]
    fn sbyte(&mut self, n: i8) {
        self.int(n as i32)
    }

    #[inline]
    fn int(&mut self, n: i32) {
        self.long(n as i64)
    }

    fn long(&mut self, n: i64);

    #[inline]
    fn byte(&mut self, n: u8) {
        self.uint(n as u32)
    }

    #[inline]
    fn uint(&mut self, n: u32) {
        self.ulong(n as u64)
    }

    fn ulong(&mut self, n: u64);

    fn single(&mut self, n: f32);

    fn double(&mut self, n: f64);

    fn bytes(&mut self, b: &[u8], align: usize);

    fn seq(&mut self) -> &mut dyn SerializerSeq;

    fn map(&mut self) -> &mut dyn SerializerMap;
}

pub trait SerializerSeq {
    fn element(&mut self, s: &dyn Serialize, c: &dyn Context);
    fn done(&mut self);
}

pub trait SerializerMap {
    fn field(&mut self, k: &str, s: &dyn Serialize, c: &dyn Context);
    fn done(&mut self);
}

/// Trait that can translate complex types based on some context
/// into serializable fragments
#[cfg(not(feature = "any-context"))]
pub trait Context {
    fn entity(&self, e: Entity) -> &dyn Serialize {
        let _ = e;
        &()
    }

    fn asset(&self, a: Asset) -> &dyn Serialize {
        let _ = a;
        &()
    }
}

#[cfg(not(feature = "any-context"))]
impl Context for () {}

#[cfg(feature = "any-context")]
pub type Context = std::any::Any;
