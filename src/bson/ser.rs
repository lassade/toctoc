use std::borrow::Cow;

use crate::buffer::Buffer;
use crate::ser::{Context, Fragment, Map, Seq, Serialize};

/// Serialize any serializable type into a BSON byte vec.
///
/// ```rust
/// use knocknoc::{bson, Serialize};
/// use knocknoc::export::hex;
///
/// #[derive(Serialize, Debug)]
/// struct Example {
///     code: u32,
///     message: String,
/// }
///
/// fn main() {
///     let example = Example {
///         code: 200,
///         message: "reminiscent of Serde".to_owned(),
///     };
///
///     let b = bson::to_bin(&example, &());
///     println!("{}", hex::encode(&b));
/// }
/// ```
pub fn to_bin<T: ?Sized + Serialize>(value: &T, context: &dyn Context) -> Vec<u8> {
    to_bin_impl(&value, context)
}

struct Serializer<'a> {
    stack: Vec<Layer<'a>>,
}

enum Layer<'a> {
    Seq(Box<dyn Seq + 'a>, usize),
    Map(Box<dyn Map + 'a>, usize),
}

impl<'a> Drop for Serializer<'a> {
    fn drop(&mut self) {
        // Drop layers in reverse order.
        while !self.stack.is_empty() {
            self.stack.pop();
        }
    }
}

// Empty document
macro_rules! empty {
    ($o:ident) => {{
        $o.write_u32(1);
        $o.write_u8(0);
    }};
}

// End document starting at some index
macro_rules! done {
    ($o:ident, $index:expr) => {{
        // End document
        let i = $index as usize;
        let l = $o.len();
        $o.iter_mut()
            .skip(i)
            .zip(&((l - i + 1) as u32).to_le_bytes()[..])
            .for_each(|(x, a)| *x = *a);
        $o.write_u8(0x00_u8);
    }};
}

fn to_bin_impl(value: &dyn Serialize, context: &dyn Context) -> Vec<u8> {
    let mut out = Buffer::new();
    let mut serializer = Serializer { stack: vec![] };
    let mut fragment = value.begin(context);
    let mut field: Option<Cow<str>> = None;

    // Root document
    out.write_u32(0);

    loop {
        // Keep type index to change it later
        let i = out.len();

        // Use null as temp type
        out.write_u8(0x0A);

        // e_name contents
        if let Some(n) = field.take() {
            out.extend_from_slice(&n.as_bytes());
        }
        out.write_u8(0x00); // c_string null terminator

        match fragment {
            Fragment::Null => {}
            Fragment::Bool(b) => {
                out[i] = 0x8;
                out.write_u8(if b { 1_u8 } else { 0_u8 });
            }
            Fragment::Str(s) => {
                out[i] = 0x02;
                out.write_u32((s.len() + 1) as u32);
                out.extend_from_slice(&s.as_bytes());
                out.write_u8(0x00); // '\0'
            }
            Fragment::U64(n) => {
                out[i] = 0x11;
                out.write_u64(n);
            }
            Fragment::I64(n) => {
                out[i] = 0x10;
                out.write_i64(n);
            }
            Fragment::F64(n) => {
                out[i] = 0x01;
                out.write_f64(n);
            }
            Fragment::Seq(mut seq) => {
                out[i] = 0x04;
                // invariant: `seq` must outlive `first`
                match careful!(seq.next() as Option<&dyn Serialize>) {
                    Some(first) => {
                        let doc = out.len();
                        out.write_u32(0);
                        serializer.stack.push(Layer::Seq(seq, doc));
                        fragment = first.begin(context);
                        continue;
                    }
                    None => empty!(out),
                }
            }
            Fragment::Map(mut map) => {
                out[i] = 0x03;
                // invariant: `map` must outlive `first`
                match careful!(map.next() as Option<(Cow<str>, &dyn Serialize)>) {
                    Some((key, first)) => {
                        let doc = out.len();
                        out.write_u32(0);
                        serializer.stack.push(Layer::Map(map, doc));
                        field = Some(key);
                        fragment = first.begin(context);
                        continue;
                    }
                    None => empty!(out),
                }
            }
            // * MOD: Format new fagment types
            // ? NOTE: These all have custom user defined types
            Fragment::U8(n) => {
                out[i] = 0x81;
                out.write_u8(n);
            }
            Fragment::I8(n) => {
                out[i] = 0x82;
                out.write_i8(n);
            }
            Fragment::U32(n) => {
                out[i] = 0x83;
                out.write_u32(n);
            }
            Fragment::I32(n) => {
                out[i] = 0x10;
                out.write_i32(n);
            }
            Fragment::F32(n) => {
                out[i] = 0x85;
                out.write_f32(n);
            }
            Fragment::Bin { bytes, align } => {
                if align == 1 {
                    out[i] = 0x05;
                    out.write_u32(bytes.len() as u32);
                    out.extend_from_slice(&bytes);
                } else {
                    out[i] = 0x8F; // Aligned data!
                    out.write_u32(bytes.len() as u32);
                    out.write_u8(align as u8);
                    let index = out.len();
                    out.write_u8(0); // Data offset
                    let offset = (out.extend_from_slice_aligned(&bytes, align) - index - 1) as u8;
                    out[index] = offset;
                }
            }
        }

        loop {
            match serializer.stack.last_mut() {
                Some(Layer::Seq(seq, doc)) => {
                    // invariant: `seq` must outlive `next`
                    match careful!(seq.next() as Option<&dyn Serialize>) {
                        Some(next) => {
                            fragment = next.begin(context);
                            break;
                        }
                        None => done!(out, *doc),
                    }
                }
                Some(Layer::Map(map, doc)) => {
                    // invariant: `map` must outlive `next`
                    match careful!(map.next() as Option<(Cow<str>, &dyn Serialize)>) {
                        Some((key, next)) => {
                            field = Some(key);
                            fragment = next.begin(context);
                            break;
                        }
                        None => done!(out, *doc),
                    }
                }
                None => {
                    done!(out, 0); // End root level document
                    return out.to_vec();
                }
            }
            serializer.stack.pop();
        }
    }
}
