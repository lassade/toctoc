use std::mem;
use std::str;

use crate::json::HEX_HINT;
use crate::de::{Deserialize, Map, Seq, Visitor, Context};
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
///     let j = r#" {"code": 200, "message": "reminiscent of Serde"} "#;
///
///     let out: Example = json::from_str(&j, &mut ())?;
///     println!("{:?}", out);
///
///     Ok(())
/// }
/// ```
pub fn from_str<T: Deserialize>(j: &str, context: &mut dyn Context) -> Result<T> {
    let mut out = None;
    let mut bytes = j.as_bytes().to_vec();
    from_str_impl(&mut bytes, T::begin(&mut out), context)?;
    out.ok_or(Error)
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

struct Deserializer<'a, 'b> {
    i: usize,
    tape: Vec<Node<'a>>,
    stack: Vec<(&'b mut dyn Visitor, Layer<'b>, usize)>,
}

impl<'a, 'b> Iterator for Deserializer<'a, 'b> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.tape.get(self.i).cloned();
        self.i += 1;
        v
    }
}

impl<'a, 'b> Drop for Deserializer<'a, 'b> {
    fn drop(&mut self) {
        // Drop layers in reverse order.
        while !self.stack.is_empty() {
            self.stack.pop();
        }
    }
}

fn from_str_impl(j: &mut [u8], mut visitor: &mut dyn Visitor, context: &mut dyn Context) -> Result<()> {
    use Node::*;
    use StaticNode::*;

    let mut de = Deserializer {
        i: 1, // Frist node is alwyas of type `Static(Null)` 
        tape: simd_json::to_tape(j).map_err(|_| Error)?,
        stack: Vec::new(),
    };

    'outer: loop {
        let tuple = match de.next() {
            Some(Static(Null)) => {
                visitor.null(context)?;
                None
            },
            Some(Static(Bool(b))) => {
                visitor.boolean(b)?;
                None
            },
            Some(Static(I64(n))) => {
                visitor.negative(n, context)?;
                None
            },
            Some(Static(U64(n))) => {
                visitor.nonnegative(n, context)?;
                None
            },
            Some(Static(F64(n))) => {
                visitor.double(n)?;
                None
            },
            Some(String(s)) => {
                if s.chars().last() == Some(HEX_HINT) {
                    let c = s.len() - 1;
                    let b = bintext::hex::decode(&s[..c]).map_err(|_| Error)?;
                    visitor.bytes(b.as_slice(), context)?;
                } else {
                    visitor.string(s, context)?;
                }
                None
            },
            Some(Array(_, finish)) => {
                let seq = careful!(visitor.seq(context)? as Box<dyn Seq>);
                Some((Layer::Seq(seq), finish))
            },
            Some(Object(_, finish)) => {
                let map = careful!(visitor.map(context)? as Box<dyn Map>);
                Some((Layer::Map(map), finish))
            },
            _ => None,
        };

        let (mut layer, mut finish) = match tuple {
            Some(t) => t,
            None => match de.stack.pop() {
                Some(frame) => {
                    visitor = frame.0;
                    (frame.1, frame.2)
                }
                None => break 'outer,
            },
        };

        // Frame ended
        while de.i >= finish {
            match &mut layer {
                Layer::Seq(seq) => seq.finish()?,
                Layer::Map(map) => map.finish()?,
            }
            match de.stack.pop() {
                Some(frame) => {
                    visitor = frame.0;
                    layer = frame.1;
                    finish = frame.2;
                },
                None => break 'outer,
            }
        }

        // Push layer back
        match layer {
            Layer::Seq(mut seq) => {
                let inner = careful!(seq.element()? as &mut dyn Visitor);
                let outer = mem::replace(&mut visitor, inner);
                de.stack.push((outer, Layer::Seq(seq), finish));
            }
            Layer::Map(mut map) => {
                let inner = {
                    let k = match de.next() {
                        Some(String(s)) => Ok(s),
                        _ => Err(Error),
                    }?;
                    careful!(map.key(k)? as &mut dyn Visitor)
                };
                let outer = mem::replace(&mut visitor, inner);
                de.stack.push((outer, Layer::Map(map), finish));
            }
        }
    }

    // All input was consumed
    match de.next() {
        Some(_) => Err(Error),
        None => Ok(()),
    }
}