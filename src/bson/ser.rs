use std::borrow::Cow;

use crate::ser::{Fragment, Map, Seq, Serialize, Context};

#[cfg(target_endian = "big")]
#[allow(unused)]
pub fn check_endianess() {
    compile_error!("bson not supported on big-endian targets, because of strings and bineary data are assumed to be little endian");
}

/// Serialize any serializable type into a JSON string.
///
/// ```rust
/// use knocknoc::{json, Serialize};
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
///     let j = json::to_bin(&example, &());
///     println!("{}", j);
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

// Change object type
macro_rules! wb {
    ($o:ident, $t:expr) => { $o.extend_from_slice(&$t.to_le_bytes()[..]); };
}

// Empty doccument
macro_rules! empty {
    ($o:ident) => { {
        wb!($o, 0x1_u32);
        wb!($o, 0x00_u8);
    } };
}

// End document starting at some index
macro_rules! done {
    ($o:ident, $index:expr) => { {
        // End document
        let i = $index as usize;
        let l = $o.len();
        $o.iter_mut().skip(i)
            .zip(&((l - i + 1) as u32).to_le_bytes()[..])
            .for_each(|(x, a)| *x = *a);
        wb!($o, 0x00_u8);
    } };
}

fn to_bin_impl(value: &dyn Serialize, context: &dyn Context) -> Vec<u8> {
    let mut out = Vec::new();
    let mut serializer = Serializer { stack: Vec::new() };
    let mut fragment = value.begin(context);
    let mut field: Option<Cow<str>> = None;
    let mut index = 0usize;

    // Root document
    let root = matches!(fragment, Fragment::Map(_) | Fragment::Seq(_));

    // Root level document
    if !root { wb!(out, 0_u32); } // TODO: write byte lenght

    loop {
        let frist = index == 0;
        index += 1;
        
         // Keep type index to change it later
        let i = out.len();

        if !frist || !root {
            // TODO: should be string or some more common type
            // Use null as temp type
            wb!(out, 0x0A_u8);

            // e_name contents
            if let Some(n) = field.take() {
                out.extend_from_slice(&n.as_bytes());
            }
            wb!(out, 0x00_u8); // c_string null terminator
        }

        match fragment {
            Fragment::Null => {},
            Fragment::Bool(b) => {
                out[i] = 0x8;
                wb!(out, if b { 1_u8 } else { 0_u8 });
            },
            Fragment::Str(s) => {
                out[i] = 0x02;
                wb!(out, (s.len() + 1) as u32);
                out.extend_from_slice(&s.as_bytes());
                wb!(out, 0x00_u8);
            },
            Fragment::U64(n) => { out[i] = 0x11; wb!(out, n); },
            Fragment::I64(n) => { out[i] = 0x10; wb!(out, n); },
            Fragment::F64(n) => { out[i] = 0x01; wb!(out, n); },
            Fragment::Seq(mut seq) => {
                if !frist || !root { out[i] = 0x04; }
                // invariant: `seq` must outlive `first`
                match careful!(seq.next() as Option<&dyn Serialize>) {
                    Some(first) => {
                        let doc = out.len();
                        wb!(out, 0_u32);
                        serializer.stack.push(Layer::Seq(seq, doc));
                        fragment = first.begin(context);
                        continue;
                    }
                    None => empty!(out),
                }
            }
            Fragment::Map(mut map) => {
                if !frist || !root { out[i] = 0x03; }
                // invariant: `map` must outlive `first`
                match careful!(map.next() as Option<(Cow<str>, &dyn Serialize)>) {
                    Some((key, first)) => {
                        let doc = out.len();
                        wb!(out, 0_u32);
                        serializer.stack.push(Layer::Map(map, doc));
                        field = Some(key);
                        fragment = first.begin(context);
                        continue;
                    }
                    None => empty!(out),
                }
            },
            // * MOD: Format new fagment types
            // ? NOTE: These all have custom user defined types
            Fragment::U8(n) => { out[i] = 0x81; wb!(out, n); },
            Fragment::I8(n) => { out[i] = 0x82; wb!(out, n); },
            Fragment::U32(n) => { out[i] = 0x83; wb!(out, n); },
            Fragment::I32(n) => { out[i] = 0x10; wb!(out, n); },
            Fragment::F32(n) => { out[i] = 0x85; wb!(out, n); },
            Fragment::Bin(b) => {
                out[i] = 0x05;
                wb!(out, b.len() as u32);
                out.extend_from_slice(&b);
            },
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
                    if !root { done!(out, 0); }
                    return out;
                }
            }
            serializer.stack.pop();
        }
    }

}