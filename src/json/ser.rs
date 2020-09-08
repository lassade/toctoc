use crate::ser::{Context, Serialize, Serializer, SerializerMap, SerializerSeq};

/// Serialize any serializable type into a JSON string.
///
/// ```rust
/// use toctoc::{json, Serialize};
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
///     let j = json::to_string(&example, &());
///     println!("{}", j);
/// }
/// ```
pub fn to_string<T: ?Sized + Serialize>(value: &T, context: &dyn Context) -> String {
    let mut json = JsonSer { out: vec![] };
    value.begin((&mut json).into(), context);
    json.done()
}

struct JsonSer {
    out: Vec<u8>,
}

impl JsonSer {
    #[inline]
    fn push(&mut self, c: u8) {
        self.out.push(c)
    }

    fn push_str(&mut self, s: &str) {
        self.out.extend_from_slice(s.as_bytes())
    }

    // Clippy false positive: https://github.com/rust-lang/rust-clippy/issues/5169
    #[allow(clippy::zero_prefixed_literal)]
    fn push_str_escaped(&mut self, value: &str) {
        self.out.push(b'"');

        let bytes = value.as_bytes();
        let mut start = 0;

        for (i, &byte) in bytes.iter().enumerate() {
            let escape = ESCAPE[byte as usize];
            if escape == 0 {
                continue;
            }

            if start < i {
                self.push_str(&value[start..i]);
            }

            match escape {
                self::BB => self.push_str("\\b"),
                self::TT => self.push_str("\\t"),
                self::NN => self.push_str("\\n"),
                self::FF => self.push_str("\\f"),
                self::RR => self.push_str("\\r"),
                self::QU => self.push_str("\\\""),
                self::BS => self.push_str("\\\\"),
                self::U => {
                    static HEX_DIGITS: [u8; 16] = *b"0123456789abcdef";
                    self.push_str("\\u00");
                    self.push(HEX_DIGITS[(byte >> 4) as usize]);
                    self.push(HEX_DIGITS[(byte & 0xF) as usize]);
                }
                _ => unreachable!(),
            }

            start = i + 1;
        }

        if start != bytes.len() {
            self.push_str(&value[start..]);
        }

        self.push(b'"');
    }

    /// **NOTE** Must guarantee that there is at least one element in `out`
    #[inline]
    unsafe fn undo_comma(&mut self) {
        let i = self.out.len() - 1;
        if *self.out.get_unchecked(i) == b',' {
            self.out.set_len(i)
        }
    }

    fn done(self) -> String {
        unsafe { String::from_utf8_unchecked(self.out) }
    }
}

impl Serializer for JsonSer {
    fn null(&mut self) {
        self.push_str("null");
    }

    fn boolean(&mut self, b: bool) {
        self.push_str(if b { "true" } else { "false" });
    }

    fn string(&mut self, s: &str) {
        self.push_str_escaped(&s);
    }

    fn long(&mut self, n: i64) {
        self.push_str(itoa::Buffer::new().format(n));
    }

    fn ulong(&mut self, n: u64) {
        self.push_str(itoa::Buffer::new().format(n));
    }

    fn double(&mut self, n: f64) {
        if n.is_finite() {
            self.push_str(ryu::Buffer::new().format_finite(n))
        } else {
            self.push_str("null")
        }
    }

    fn seq(&mut self) -> &mut dyn SerializerSeq {
        self.push(b'[');
        self
    }

    fn map(&mut self) -> &mut dyn SerializerMap {
        self.push(b'{');
        self
    }

    fn single(&mut self, n: f32) {
        if n.is_finite() {
            self.push_str(ryu::Buffer::new().format_finite(n))
        } else {
            self.push_str("null")
        }
    }

    fn bytes(&mut self, b: &[u8], a: usize) {
        self.push_str("\"#");
        // Extra padding bytes for maneuvering, to ensure alignment
        for _ in 0..(a / 2) {
            self.push_str("--");
        }
        self.push_str(&bintext::hex::encode(b.as_ref()));
        self.push(b'"');
    }
}

impl SerializerSeq for JsonSer {
    fn element(&mut self, s: &dyn Serialize, c: &dyn Context) {
        s.begin(self.into(), c);
        self.push(b',');
    }

    fn done(&mut self) {
        unsafe {
            self.undo_comma();
        }
        self.push(b']');
    }
}

impl SerializerMap for JsonSer {
    fn field(&mut self, f: &str, s: &dyn Serialize, c: &dyn Context) {
        self.push(b'\"');
        self.push_str(f);
        self.push_str("\":");
        s.begin(self.into(), c);
        self.push(b',');
    }

    fn done(&mut self) {
        unsafe {
            self.undo_comma();
        }
        self.push(b'}');
    }
}

const BB: u8 = b'b'; // \x08
const TT: u8 = b't'; // \x09
const NN: u8 = b'n'; // \x0A
const FF: u8 = b'f'; // \x0C
const RR: u8 = b'r'; // \x0D
const QU: u8 = b'"'; // \x22
const BS: u8 = b'\\'; // \x5C
const U: u8 = b'u'; // \x00...\x1F except the ones above

// Lookup table of escape sequences. A value of b'x' at index i means that byte
// i is escaped as "\x" in JSON. A value of 0 means that byte i is not escaped.
#[rustfmt::skip]
static ESCAPE: [u8; 256] = [
    //  1   2   3   4   5   6   7   8   9   A   B   C   D   E   F
    U,  U,  U,  U,  U,  U,  U,  U, BB, TT, NN,  U, FF, RR,  U,  U, // 0
    U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U,  U, // 1
    0,  0, QU,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 2
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 3
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 4
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, BS,  0,  0,  0, // 5
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 6
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 7
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 8
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // 9
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // A
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // B
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // C
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // D
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // E
    0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0,  0, // F
];
