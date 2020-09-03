use std::mem;

use crate::de::{Context, Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use simd_json::{Node, StaticNode};

/// Deserialize a JSON string into any deserializable type.
///
/// ```rust
/// use knocknoc::{json, Deserialize};
///
/// #[derive(Deserialize, Debug)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() -> knocknoc::Result<()> {
///     let mut j = r#" {"code": 200, "message": "reminiscent of Serde"} "#.to_string();
///
///     let out: Example = json::from_str(&mut j, &mut ())?;
///     println!("{:?}", out);
///
///     Ok(())
/// }
/// ```
pub fn from_str<'de, T: Deserialize<'de>>(json: &'de mut str, ctx: &mut dyn Context) -> Result<T> {
    let mut out = None;
    let mut de = JsonDe::new(json)?;
    de.visit(T::begin(&mut out), ctx)?;
    out.ok_or_else(Error::unknown)
}

struct JsonDe<'de> {
    index: usize,
    tape: Vec<Node<'de>>,
}

impl<'de> JsonDe<'de> {
    fn new(json: &'de mut str) -> Result<Self> {
        Ok(Self {
            index: 1, // First node is always of type `Static(Null)`,
            tape: simd_json::to_tape(unsafe { json.as_bytes_mut() })
                .map_err(|err| Error::generic(err.to_string()))?,
        })
    }

    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()> {
        use Node::*;
        use StaticNode::*;

        match Iterator::next(self) {
            Some(Static(Null)) => v.null(c)?,
            Some(Static(Bool(b))) => v.boolean(b)?,
            Some(Static(I64(n))) => v.negative(n, c)?,
            Some(Static(U64(n))) => v.nonnegative(n, c)?,
            Some(Static(F64(n))) => v.double(n)?,
            Some(String(s)) => {
                // ! FIXME Not good for all occasions
                if s.starts_with('#') {
                    let mut a = 0;
                    for ch in s.as_bytes().iter().skip(1) {
                        if *ch != b'-' {
                            break;
                        }
                        a += 1;
                    }

                    // TODO: Is there a better way?
                    #[allow(mutable_transmutes)]
                    let b = unsafe {
                        let s = mem::transmute(s);
                        bintext::hex::decode_aligned(s, a + 1, a.max(1))
                            .map_err(|err| Error::generic(err.to_string()))?
                    };

                    v.bytes(b, c)?;
                } else {
                    v.string(s, c)?;
                }
            }
            Some(Array(_, e)) => {
                v.seq(&mut Stack { e, de: self }, c)?;
            }
            Some(Object(_, e)) => {
                v.map(&mut Stack { e, de: self }, c)?;
            }
            _ => {}
        }
        Ok(())
    }
}

struct Stack<'a, 'de: 'de> {
    e: usize,
    de: &'a mut JsonDe<'de>,
}

impl<'a, 'de: 'de> Seq<'de> for Stack<'a, 'de> {
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<bool> {
        if self.de.index < self.e {
            JsonDe::visit(self.de, v, c)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a, 'de: 'de> Map<'de> for Stack<'a, 'de> {
    fn next(&mut self) -> Result<Option<&'de str>> {
        use Node::*;

        if self.de.index < self.e {
            if let Some(String(s)) = self.de.next() {
                Ok(Some(s))
            } else {
                Err(Error::expecting("field name"))?
            }
        } else {
            Ok(None)
        }
    }

    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()> {
        JsonDe::visit(self.de, v, c)
    }
}

impl<'de> Iterator for JsonDe<'de> {
    type Item = Node<'de>;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.tape.get(self.index).cloned();
        self.index += 1;
        v
    }
}
