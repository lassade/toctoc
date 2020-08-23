use std::borrow::Cow;

use crate::bytes::guess_align_of;
use crate::de::{self, Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use crate::json::{Array, Number, Object};
use crate::ser::{self, Serialize};
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
    fn begin(&self, v: ser::Visitor, c: &dyn ser::Context) -> ser::Done {
        match self {
            Value::Null => v.null(),
            Value::Bool(b) => v.boolean(*b),
            Value::Number(Number::U64(n)) => v.ulong(*n),
            Value::Number(Number::I64(n)) => v.long(*n),
            Value::Number(Number::F32(n)) => v.single(*n), // * MOD: f32 support
            Value::Number(Number::F64(n)) => v.double(*n),
            Value::Binary { bytes, align } => v.bytes(bytes, *align),
            Value::String(s) => v.string(s),
            Value::Array(array) => {
                let mut seq = v.seq();
                for e in array.into_iter() {
                    seq = seq.element(e, c);
                }
                seq.done()
            }
            Value::Object(object) => {
                let mut map = v.map();
                for (k, e) in object.into_iter() {
                    map = map.field(k, e, c);
                }
                map.done()
            }
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

            fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn de::Context) -> Result<()> {
                let mut array = Array::new();
                let mut element: Option<Value> = None;
                while s.visit(Place::new(&mut element), c)? {
                    element.take().map(|e| array.push(e));
                }
                self.out = Some(Value::Array(array));
                Ok(())
            }

            fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn de::Context) -> Result<()> {
                let mut object = Object::new();
                let mut value: Option<Value> = None;
                while let Some(key) = m.next()? {
                    m.visit(Place::new(&mut value), c)?;
                    value.take().map(|v| object.insert(key.to_owned(), v));
                }
                self.out = Some(Value::Object(object));
                Ok(())
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

        // ! FIXME: Highly unsafe this will remove
        Place::new(out)
    }
}

impl<'de> PartialEq<Value<'de>> for Value<'de> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(left), Value::Bool(right)) => left == right,
            (Value::Number(left), Value::Number(right)) => left == right,
            (Value::String(left), Value::String(right)) => left == right,
            (
                Value::Binary {
                    bytes: left0,
                    align: left1,
                },
                Value::Binary {
                    bytes: right0,
                    align: right1,
                },
            ) => left0 == right0 && left1 == right1,
            (Value::Array(left), Value::Array(right)) => left == right,
            (Value::Object(left), Value::Object(right)) => left == right,
            _ => false,
        }
    }
}

impl<'de> Value<'de> {
    /// Converts a hex string into binary data
    pub fn from_hex(&mut self) -> Result<()> {
        match self {
            Value::String(ref s) => {
                *self = Value::Binary {
                    bytes: Cow::Owned(bintext::hex::decode_noerr(s).map_err(|_| Error)?),
                    align: 1, // Lowest possible alignment rank
                }
            }
            _ => {
                Err(Error)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    #[test]
    fn many_cases() {
        let cases = &[
            (Value::Null, "null"),
            (Value::Number(Number::I64(-1)), "-1"),
            (Value::Number(Number::F64(1.0)), "1.0"),
            (
                Value::Array({
                    let mut array = Array::new();
                    array.push(Value::Number(Number::U64(1)));
                    array.push(Value::Number(Number::U64(2)));
                    array
                }),
                "[1,2]",
            ),
            (
                Value::Object({
                    let mut object = Object::new();
                    object.insert("key".to_string(), Value::Number(Number::U64(2)));
                    object
                }),
                r#"{"key":2}"#,
            ),
        ];

        for (val, json) in cases {
            let mut json = json.to_string();
            let actual: Value = json::from_str(&mut json, &mut ()).unwrap();
            assert_eq!(val, &actual);
        }

        for (val, json) in cases {
            let actual = json::to_string(val, &());
            assert_eq!(json, &actual);
        }
    }
}
