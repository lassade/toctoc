use paste::paste;
use std::io::Read;
use std::mem::MaybeUninit;
use std::str;

use crate::de::{Context, Deserialize, Map, Seq, Visitor};
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
pub fn from_bin<'de, T: Deserialize<'de>>(b: &'de [u8], ctx: &mut dyn Context) -> Result<T> {
    let mut out = None;
    let bson = BsonDe::new(b)?;
    bson.deserialize(T::begin(&mut out), ctx)?;
    out.ok_or(Error.into())
}

struct BsonDe<'de> {
    buffer: &'de [u8],
    index: usize,
    ty: u8,
    key: &'de str,
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

/// Provides various functions to read bytes from the inner buffer
/// and interpreting as little endian bytes many primitive types
impl<'de> BsonDe<'de> {
    fn new(buffer: &'de [u8]) -> Result<Self> {
        // The buffer must be aligned with 4
        if buffer.as_ptr().align_offset(4) != 0 {
            Err(Error)?
        }

        let de = Self {
            buffer,
            index: 0,
            ty: 0,
            key: "",
        };

        Ok(de)
    }

    fn deserialize(mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()> {
        // Root document size
        self.read_u32()? as usize;

        self.next()?;
        self.visit(v, c)?;

        // Document done and all input was consumed
        if self.read_u8()? != 0 || self.buffer.len() != 0 {
            Err(Error)
        } else {
            Ok(())
        }
    }

    fn next(&mut self) -> Result<()> {
        self.ty = self.read_u8()?;
        self.key = self.read_cstring()?; // Always empty
        Ok(())
    }

    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()> {
        match self.ty {
            0x0A => {
                v.null(c)?;
            }
            0x08 => {
                let b = self.read_u8()? != 0;
                v.boolean(b)?;
            }
            0x12 => {
                let n = self.read_i64()?;
                v.negative(n, c)?;
            }
            0x11 => {
                let n = self.read_u64()?;
                v.nonnegative(n, c)?;
            }
            0x01 => {
                let n = self.read_f64()?;
                v.double(n)?;
            }
            0x05 => {
                // Binary
                let size = self.read_u32()?;
                let b = self.read_bytes(size as usize)?;
                v.bytes(b, c)?;
            }
            0x8F => {
                // Aligned data!
                let size = self.read_u32()?;
                let align: u8 = self.read_u8()?;
                let offset = self.read_u8()?;
                self.read_bytes(offset as usize)?;
                let b = self.read_bytes(size as usize)?;

                // Error if isn't aligned or not valid align
                if !align.is_power_of_two()
                    || align == 0
                    || b.as_ptr().align_offset(align as usize) != 0
                {
                    Err(Error)?
                }

                v.bytes(b, c)?;
            }
            0x02 => {
                // Utf8 String
                let size = self.read_u32()?;
                let bytes = self.read_bytes((size - 1) as usize)?;
                // TODO: Maybe implement the `lookup4` algorithm
                if !faster_utf8_validator::validate(bytes) {
                    Err(Error)?
                }
                let s = unsafe { str::from_utf8_unchecked(bytes) };
                self.read_u8()?; // read the '\0'
                v.string(s, c)?;
            }
            0x81 => {
                let n = self.read_u8()?;
                v.nonnegative(n as u64, c)?;
            }
            0x82 => {
                let n = self.read_i8()?;
                v.negative(n as i64, c)?;
            }
            0x83 => {
                let n = self.read_u32()?;
                v.nonnegative(n as u64, c)?;
            }
            0x10 => {
                let n = self.read_i32()?;
                v.negative(n as i64, c)?;
            }
            0x85 => {
                let n = self.read_f32()?;
                v.single(n)?;
            }
            0x04 => {
                let size = self.read_i32()?;
                // Subtract 4 bytes of the size it self and 1 of '\0' (end document)
                let e = size as usize + self.index - 5;
                let mut stack = Stack { e, de: self };
                v.seq(&mut stack, c)?;
                self.read_u8()?; // '\0' (end document)
                self.index = e + 1; // No matter what skip the entire document
            }
            0x03 => {
                let size = self.read_i32()?;
                let e = size as usize + self.index - 5;
                let mut stack = Stack { e, de: self };
                v.map(&mut stack, c)?;
                self.read_u8()?; // '\0' (end document)
                self.index = e + 1; // No matter what skip the entire document
            }
            _ => Err(Error)?, // Unknown type
        }
        Ok(())
    }

    read_byte_impl!(u8, i8, u32, i32, u64, i64, f32, f64);

    fn read_bytes(&mut self, length: usize) -> Result<&'de [u8]> {
        if length < self.buffer.len() {
            let (buf, rem) = self.buffer.split_at(length);
            self.buffer = rem;
            self.index += length;
            Ok(buf)
        } else {
            Err(Error)
        }
    }

    /// Reads a sequence of bytes until find a '\0' then return it as str
    fn read_cstring(&mut self) -> Result<&'de str> {
        for (mut i, byte) in self.buffer.iter().copied().enumerate() {
            if byte != 0 {
                continue;
            }
            if !byte.is_ascii() {
                break;
            }

            unsafe {
                let ptr = self.buffer.as_ptr();
                let buf = std::slice::from_raw_parts(ptr, i);

                // Plus 1 because we don't need the '\0' string terminator
                i += 1;
                self.buffer = std::slice::from_raw_parts(ptr.add(i), self.buffer.len() - i);
                self.index += i;

                // Not an utf8 string, but an ascii sequence
                return Ok(str::from_utf8_unchecked(buf));
            }
        }

        Err(Error)
    }
}

struct Stack<'a, 'de: 'de> {
    e: usize,
    de: &'a mut BsonDe<'de>,
}

impl<'a, 'de: 'de> Seq<'de> for Stack<'a, 'de> {
    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<bool> {
        if self.de.index < self.e {
            self.de.next()?;
            self.de.visit(v, c)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<'a, 'de: 'de> Map<'de> for Stack<'a, 'de> {
    fn next(&mut self) -> Result<Option<&'de str>> {
        if self.de.index < self.e {
            self.de.next()?;
            Ok(Some(self.de.key))
        } else {
            Ok(None)
        }
    }

    fn visit(&mut self, v: &mut dyn Visitor<'de>, c: &mut dyn Context) -> Result<()> {
        self.de.visit(v, c)
    }
}
