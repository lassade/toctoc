use std::borrow::Cow;
use std::mem;

use crate::bytes::guess_align_of;
use crate::de::{self, Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use crate::json::{Array, Number, Object};
use crate::private;
use crate::ser::{self, Fragment, Serialize};
use crate::Place;

/// Any valid JSON value.
///
/// This type has a non-recursive drop implementation so it is safe to build
/// arbitrarily deeply nested instances.
///
/// ```rust
/// use knocknoc::json::{Array, Value};
///
/// let mut value = Value::Null;
/// for _ in 0..100000 {
///     let mut array = Array::new();
///     array.push(value);
///     value = Value::Array(array);
/// }
/// // no stack overflow when `value` goes out of scope
/// ```
#[derive(Clone, Debug)]
pub enum Value<'a> {
    Null,
    Bool(bool),
    Number(Number),
    String(Cow<'a, str>),
    // * MOD: Byte support
    Binary {
        /// Unaligned binary data
        bytes: Cow<'a, [u8]>,
        /// Desired bytes alignment
        align: usize,
    },
    Array(Array<'a>),
    Object(Object<'a>),
}

impl<'a> Default for Value<'a> {
    /// The default value is null.
    fn default() -> Self {
        Value::Null
    }
}

impl<'a> Serialize for Value<'a> {
    fn begin(&self, _: &dyn ser::Context) -> Fragment {
        match self {
            Value::Null => Fragment::Null,
            Value::Bool(b) => Fragment::Bool(*b),
            Value::Number(Number::U64(n)) => Fragment::U64(*n),
            Value::Number(Number::I64(n)) => Fragment::I64(*n),
            Value::Number(Number::F32(n)) => Fragment::F32(*n), // * MOD: f32 support
            Value::Number(Number::F64(n)) => Fragment::F64(*n),
            Value::Binary { bytes, align } => Fragment::Bin {
                bytes: Cow::Borrowed(bytes),
                align: *align,
            }, // * MOD: binary data support
            Value::String(s) => Fragment::Str(Cow::Borrowed(s)),
            Value::Array(array) => private::stream_slice(array),
            Value::Object(object) => private::stream_object(object),
        }
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for Value<'a> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'a, 'de: 'a> Visitor<'de> for Place<Value<'a>> {
            fn null(&mut self, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Null);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(Value::Bool(b));
                Ok(())
            }

            fn string(&mut self, s: &'de str, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::String(Cow::Borrowed(s)));
                Ok(())
            }

            fn negative(&mut self, n: i64, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Number(Number::I64(n)));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Number(Number::U64(n)));
                Ok(())
            }

            fn double(&mut self, n: f64) -> Result<()> {
                self.out = Some(Value::Number(Number::F64(n)));
                Ok(())
            }

            fn seq<'seq>(&'seq mut self) -> Result<Box<dyn Seq<'de> + 'seq>>
            where
                'de: 'seq,
            {
                Ok(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: Array::new(),
                    element: None,
                }))
            }

            fn map<'map>(&'map mut self) -> Result<Box<dyn Map<'de> + 'map>>
            where
                'de: 'map,
            {
                Ok(Box::new(ObjectBuilder {
                    out: &mut self.out,
                    object: Object::new(),
                    key: None,
                    value: None,
                }))
            }

            fn single(&mut self, n: f32) -> Result<()> {
                self.out = Some(Value::Number(Number::F32(n)));
                Ok(())
            }

            fn bytes(&mut self, b: &'de [u8], _: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Binary {
                    bytes: Cow::Borrowed(b),
                    align: guess_align_of(b.as_ptr()),
                });
                Ok(())
            }
        }

        struct ArrayBuilder<'arr, 'a: 'arr> {
            out: &'arr mut Option<Value<'a>>,
            array: Array<'a>,
            element: Option<Value<'a>>,
        }

        impl<'arr, 'a: 'arr> ArrayBuilder<'arr, 'a> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.array.push(e);
                }
            }
        }

        impl<'arr, 'a: 'arr, 'de: 'a> Seq<'de> for ArrayBuilder<'arr, 'a> {
            fn element(&mut self) -> Result<&mut dyn Visitor<'de>> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self, _: &'_ mut dyn de::Context) -> Result<()> {
                self.shift();
                *self.out = Some(Value::Array(mem::replace(&mut self.array, Array::new())));
                Ok(())
            }
        }

        struct ObjectBuilder<'obj, 'a: 'obj> {
            out: &'obj mut Option<Value<'a>>,
            object: Object<'a>,
            key: Option<String>,
            value: Option<Value<'a>>,
        }

        impl<'obj, 'a: 'obj> ObjectBuilder<'obj, 'a> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.object.insert(k, v);
                }
            }
        }

        impl<'obj, 'a: 'obj, 'de: 'a> Map<'de> for ObjectBuilder<'obj, 'a> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'de>> {
                self.shift();
                self.key = Some(k.to_owned());
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self, _: &'_ mut dyn de::Context) -> Result<()> {
                self.shift();
                *self.out = Some(Value::Object(mem::replace(&mut self.object, Object::new())));
                Ok(())
            }
        }

        // ! FIXME: Highly unsafe this will remove
        Place::new(out)
    }
}

impl<'de> Value<'de> {
    /// Converts a hex string into binary data
    pub fn from_hex(&mut self) -> Result<()> {
        match self {
            Value::String(ref s) => {
                *self = Value::Binary {
                    bytes: Cow::Owned(bintext::hex::decode_no(s).map_err(|_| Error)?),
                    align: 1, // Lowest possible alignment rank
                }
            }
            _ => {
                return Err(Error);
            }
        }
        Ok(())
    }
}
