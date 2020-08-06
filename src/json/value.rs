use std::borrow::Cow;
use std::mem;

use crate::de::{self, Deserialize, Map, Seq, Visitor};
use crate::error::{Result, Error};
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
pub enum Value<'i> {
    Null,
    Bool(bool),
    Number(Number),
    String(Cow<'i, str>),
    Binary(Cow<'i, [u8]>), // * MOD: Byte support
    Array(Array<'i>),
    Object(Object<'i>),
}

impl<'i> Default for Value<'i> {
    /// The default value is null.
    fn default() -> Self {
        Value::Null
    }
}

impl<'i> Serialize for Value<'i> {
    fn begin(&self, _c: &dyn ser::Context) -> Fragment {
        match self {
            Value::Null => Fragment::Null,
            Value::Bool(b) => Fragment::Bool(*b),
            Value::Number(Number::U64(n)) => Fragment::U64(*n),
            Value::Number(Number::I64(n)) => Fragment::I64(*n),
            Value::Number(Number::F32(n)) => Fragment::F32(*n), // * MOD: f32 support
            Value::Number(Number::F64(n)) => Fragment::F64(*n),
            Value::Binary(b) => Fragment::Bin(Cow::Borrowed(b)), // * MOD: binary data support
            Value::String(s) => Fragment::Str(Cow::Borrowed(s)),
            Value::Array(array) => private::stream_slice(array),
            Value::Object(object) => private::stream_object(object),
        }
    }
}

impl<'i> Deserialize<'i> for Value<'i> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<Value<'i>> {
            fn null(&mut self, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Null);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(Value::Bool(b));
                Ok(())
            }

            fn string(&mut self, s: &'i str, _c: &mut dyn de::Context) -> Result<()> {
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

            fn seq<'a>(&'a mut self) -> Result<Box<dyn Seq<'i> + 'a>> 
            where
                'i: 'a
            {
                Ok(Box::new(ArrayBuilder {
                    out: &mut self.out,
                    array: Array::new(),
                    element: None,
                }))
            }

            fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'i> + 'a>>
            where
                'i: 'a
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
        
            fn bytes(&mut self, b: &'i [u8], _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Value::Binary(Cow::Borrowed(b)));
                Ok(())
            }
        }

        struct ArrayBuilder<'a, 'i> {
            out: &'a mut Option<Value<'i>>,
            array: Array<'i>,
            element: Option<Value<'i>>,
        }

        impl<'a, 'i> ArrayBuilder<'a, 'i> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.array.push(e);
                }
            }
        }

        impl<'a, 'i> Seq<'i> for ArrayBuilder<'a, 'i> {
            fn element(&mut self) -> Result<&mut dyn Visitor<'i>> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self, _: &'_ mut dyn de::Context) -> Result<()> {
                self.shift();
                *self.out = Some(Value::Array(mem::replace(&mut self.array, Array::new())));
                Ok(())
            }
        }

        struct ObjectBuilder<'a, 'i> {
            out: &'a mut Option<Value<'i>>,
            object: Object<'i>,
            key: Option<String>,
            value: Option<Value<'i>>,
        }

        impl<'a, 'i> ObjectBuilder<'a, 'i> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.object.insert(k, v);
                }
            }
        }

        impl<'a, 'i> Map<'i> for ObjectBuilder<'a, 'i> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'i>> {
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

impl<'i> Value<'i> {
    /// Converts a hex string into binary data
    pub fn from_hex(&mut self) -> Result<()> {
        match self {
            Value::String(ref s) =>
                *self = Value::Binary(Cow::Owned(bintext::hex::decode_no(s).map_err(|_| Error)?)),
            _ => { return Err(Error); }
        }
        Ok(())
    }
}