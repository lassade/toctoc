use std::str;
use std::io::Read;
use std::mem;
use std::mem::MaybeUninit;
use paste::paste;

use crate::de::{Deserialize, Map, Seq, Visitor, Context};
use crate::error::{Error, Result};

/// Deserialize a BSON byte vec into any deserializable type.
///
/// ```rust
/// use knocknoc::{bson, Deserialize};
/// use knocknoc::export::hex;
///
/// #[derive(Deserialize, Debug)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() -> knocknoc::Result<()> {
///     let h = "3800000003003100000083636f646500c8000000026d\
///              657373616765001500000072656d696e697363656e74\
///              206f66205365726465000000";
///
///     let b = hex::decode(h).unwrap();
///     let out: Example = bson::from_bin(&b, &mut ())?;
///     println!("{:?}", out);
///
///     Ok(())
/// }
/// ```
pub fn from_bin<T: Deserialize>(b: &[u8], ctx: &mut dyn Context) -> Result<T> {
    let mut out = None;
    from_bin_impl(b, T::begin(&mut out), ctx)?;
    out.ok_or(Error)
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>),
    Map(Box<dyn Map + 'a>),
}

struct Deserializer<'a, 'b> {
    buffer: &'a [u8],
    index: usize,
    stack: Vec<(&'b mut dyn Visitor, Layer<'b>, usize)>,
}

macro_rules! read_byte_impl {
    ($($t:ty),*) => {
        $(paste! {
            fn [<read_ $t>] (&mut self) -> Result<$t> {
                let mut a = unsafe {
                    MaybeUninit::<[u8; std::mem::size_of::<$t>()]>::uninit()
                        .assume_init()
                };
                self.buffer.read_exact(&mut a).map_err(|_| Error)?;
                self.index += std::mem::size_of::<$t>();
                Ok($t::from_le_bytes(a))
            }
        })*
    };
}

/// Provides varius functions to read bytes from the inner buffer
/// and interpreting as little endian bytes many primitive types
impl<'a, 'b>  Deserializer<'a, 'b> {
    read_byte_impl!(u8, i8, u32, i32, u64, i64, f32, f64);

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8]> {
        let i = length - 1;
        if i < self.buffer.len() {
            let (buf, rem) = self.buffer.split_at(i);
            self.buffer = rem;
            self.index += i;
            Ok(buf)
        } else {
            Err(Error)
        }
    }

    /// Reads a sequence of bytes until find a '\0' then return it as str
    fn read_cstring(&mut self) -> Result<&'a str> {
        for (mut i, byte) in self.buffer.iter().copied().enumerate() {
            if byte != 0 { continue }
            if !byte.is_ascii() { break; }

            unsafe { 
                let ptr = self.buffer.as_ptr();
                let buf = std::slice::from_raw_parts(ptr, i);

                // Plus 1 because we don't need the '\0' string terminator
                i += 1;
                self.buffer = std::slice::from_raw_parts(
                    ptr.add(i), 
                    self.buffer.len() - i);
                self.index += i;

                // Not an utf8 string, but an ascii sequence
                return Ok(str::from_utf8_unchecked(buf));
            }
        }

        Err(Error)
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

fn from_bin_impl(buffer: &[u8], mut visitor: &mut dyn Visitor, context: &mut dyn Context) -> Result<()> {
    let mut de = Deserializer {
        buffer,
        index: 0,
        stack: Vec::new()
    };

    de.read_u32()?; // Document size
    let mut _type = de.read_u8()?;
    let mut e_name;
    de.read_cstring()?; // Always empty

    'outer: loop {
        let tuple = match _type {
            0x0A => {
                visitor.null(context)?;
                None
            },
            0x08 => {
                let b = de.read_u8()? != 0;
                visitor.boolean(b)?;
                None
            },
            0x12 => {
                let n = de.read_i64()?;
                visitor.negative(n, context)?;
                None
            },
            0x11 => {
                let n = de.read_u64()?;
                visitor.nonnegative(n, context)?;
                None
            },
            0x01 => {
                let n = de.read_f64()?;
                visitor.double(n)?;
                None
            },
            0x05 => { // Binary
                let size = de.read_u32()?;
                let b = de.read_bytes(size as usize)?;
                visitor.bytes(b, context)?;
                None
            },
            0x02 => {
                let size = de.read_u32()?;
                // TODO: Maybe implement the `lookup4` algorimth
                let bytes = de.read_bytes(size as usize)?;
                if !faster_utf8_validator::validate(bytes) {
                    Err(Error)?
                }
                let s = unsafe { str::from_utf8_unchecked(bytes) };
                de.read_u8()?;
                visitor.string(s, context)?;
                None
            },
            0x81 => {
                let n = de.read_u8()?;
                visitor.nonnegative(n as u64, context)?;
                None
            },
            0x82 => {
                let n = de.read_i8()?;
                visitor.negative(n as i64, context)?;
                None
            },
            0x83 => {
                let n = de.read_u32()?;
                visitor.nonnegative(n as u64, context)?;
                None
            },
            0x10 => {
                let n = de.read_i32()?;
                visitor.negative(n as i64, context)?;
                None
            },
            0x85 => {
                let n = de.read_f32()?;
                visitor.single(n)?;
                None
            },
            0x04 => {
                let size = de.read_i32()?;
                // Subtract 4 bytes of the size it self and 1 of '\0' (end document)
                let size = size as usize + de.index - 5;
                let seq = careful!(visitor.seq(context)? as Box<dyn Seq>);
                Some((Layer::Seq(seq), size))
            },
            0x03 => {
                let size = de.read_i32()?;
                let size = size as usize + de.index - 5;
                let map = careful!(visitor.map(context)? as Box<dyn Map>);
                Some((Layer::Map(map), size))
            },
            _ => Err(Error)?,
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

        // Document ended
        while de.index >= finish {
            if de.read_u8()? != 0 { Err(Error)? }

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

        // Update type and e_name
        _type = de.read_u8()?;
        e_name = de.read_cstring()?;

        // Push layer back
        match layer {
            Layer::Seq(mut seq) => {
                let inner = careful!(seq.element()? as &mut dyn Visitor);
                let outer = mem::replace(&mut visitor, inner);
                de.stack.push((outer, Layer::Seq(seq), finish));
            }
            Layer::Map(mut map) => {
                let inner = {
                    careful!(map.key(e_name)? as &mut dyn Visitor)
                };
                let outer = mem::replace(&mut visitor, inner);
                de.stack.push((outer, Layer::Map(map), finish));
            }
        }
    }

    // Document done and all input was consumed
    if de.read_u8()? != 0 || de.buffer.len() != 0 {
        Err(Error)
    } else {
        Ok(())
    }
}