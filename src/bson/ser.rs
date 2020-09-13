use crate::buffer::Buffer;
use crate::ser::{Context, MapTrait, Return, SeqTrait, Serialize, SerializerTrait, VisitorTrait};

/// Serialize any serializable type into a BSON byte vec.
///
/// ```rust
/// use toctoc::{bson, Serialize};
/// use toctoc::export::hex;
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
///     let b = bson::to_bin(&example, &mut ());
///     println!("{}", hex::encode(&b));
/// }
/// ```
pub fn to_bin<T: Serialize>(value: &T, context: &mut dyn Context) -> Vec<u8> {
    let mut bson = BsonSer::new();
    match bson.serialize(value, context) {
        Return::Text(_) => unreachable!(),
        Return::Binary(b) => b,
    }
}

pub struct BsonSer<'a> {
    buffer: Buffer,
    doc: Vec<usize>,
    field: Option<&'a str>,
    /// Alignment field metadata
    align: usize,
    dirty: bool,
}

impl<'a> BsonSer<'a> {
    pub fn new() -> Self {
        let mut bson = Self {
            buffer: Buffer::new(),
            doc: vec![],
            field: None,
            align: Buffer::ALIGNMENT, // Default alignment
            dirty: false,
        };

        // Root document
        bson.begin_doc();

        if cfg!(feature = "higher-rank-alignment") {
            // Serialize the alignment requirement as the first document field
            bson.field = Some("align");
            bson.uint(bson.align as u32);
            assert_eq!(bson.buffer.len(), 15); // Make sure the alignment is the 11th byte on buffer
        }

        bson
    }

    fn element(&mut self, ty: u8) -> usize {
        // Keep type index to change it later
        let i = self.buffer.len();

        // Use null as temp type
        self.buffer.write_u8(ty);

        // e_name contents
        if let Some(n) = self.field.take() {
            self.buffer.extend_from_slice(&n.as_bytes());
        }
        self.buffer.write_u8(0x00); // c_string null terminator
        i
    }

    fn begin_doc(&mut self) {
        let i = self.buffer.len();
        self.buffer.write_u32(0);
        self.doc.push(i);
    }

    // End document starting at some index
    fn end_doc(&mut self) {
        // End document
        let i = self.doc.pop().unwrap();
        let l = self.buffer.len();
        self.buffer.replace_u32(i, (l - i + 1) as u32);
        self.buffer.write_u8(0x00_u8);
    }
}

impl<'a> SerializerTrait for BsonSer<'a> {
    fn serialize(&mut self, s: &dyn Serialize, c: &mut dyn Context) -> Return {
        // Clean up
        // ? NOTE: This is needed because the bson needs to be decorated with a root document
        if self.dirty {
            let mut s = Self::new();
            std::mem::swap(self, &mut s);
        }

        s.begin(self.into(), c);
        self.end_doc(); // End root level document
        self.dirty = true; // Needs a clean up before reuse!

        let mut b = Buffer::new();
        std::mem::swap(&mut self.buffer, &mut b);

        Return::Binary(b.to_vec())
    }
}

impl<'a> VisitorTrait for BsonSer<'a> {
    fn null(&mut self) {
        self.element(0x0A);
    }

    fn boolean(&mut self, b: bool) {
        self.element(0x8);
        self.buffer.write_u8(if b { 1_u8 } else { 0_u8 });
    }

    fn string(&mut self, s: &str) {
        self.element(0x02);
        self.buffer.write_u32((s.len() + 1) as u32);
        self.buffer.extend_from_slice(&s.as_bytes());
        self.buffer.write_u8(0x00); // '\0'
    }

    fn byte(&mut self, n: u8) {
        self.element(0x81);
        self.buffer.write_u8(n);
    }

    fn sbyte(&mut self, n: i8) {
        self.element(0x82);
        self.buffer.write_i8(n);
    }

    fn int(&mut self, n: i32) {
        self.element(0x10);
        self.buffer.write_i32(n);
    }

    fn uint(&mut self, n: u32) {
        self.element(0x83);
        self.buffer.write_u32(n);
    }

    fn long(&mut self, n: i64) {
        self.element(0x10);
        self.buffer.write_i64(n);
    }

    fn ulong(&mut self, n: u64) {
        self.element(0x11);
        self.buffer.write_u64(n);
    }

    fn single(&mut self, n: f32) {
        self.element(0x85);
        self.buffer.write_f32(n);
    }

    fn double(&mut self, n: f64) {
        self.element(0x01);
        self.buffer.write_f64(n);
    }

    fn bytes(&mut self, b: &[u8], a: usize) {
        if a == 1 {
            self.element(0x05);
            self.buffer.write_u32(b.len() as u32);
            self.buffer.extend_from_slice(&b);
        } else {
            self.element(0x8F); // Aligned data!
            self.buffer.write_u32(b.len() as u32);
            self.buffer.write_u32(a as u32);

            if a > self.align {
                if cfg!(feature = "higher-rank-alignment") {
                    // Buffer must have a higher align requirement
                    self.align = a;
                    self.buffer.replace_u32(11, a as u32);
                } else {
                    // Sorry, a panic now is better than later figuring the data can't be properly read
                    unimplemented!(
                        "{} is higher alignment than the default {} isn't supported,\
                        consider enable the `higher-rank-alignment` feature",
                        a,
                        Buffer::ALIGNMENT
                    )
                }
            }

            self.buffer.write_u32(0); // Data offset
            let index = self.buffer.len();
            let offset = self.buffer.extend_from_slice_aligned(&b, a) - index;
            self.buffer
                .replace_u32(index - std::mem::size_of::<u32>(), offset as u32);
        }
    }

    fn seq(&mut self) -> &mut dyn SeqTrait {
        self.element(0x04);
        self.begin_doc();
        self
    }

    fn map(&mut self) -> &mut dyn MapTrait {
        self.element(0x03);
        self.begin_doc();
        self
    }
}

impl<'a> SeqTrait for BsonSer<'a> {
    fn element(&mut self, s: &dyn Serialize, c: &mut dyn Context) {
        s.begin(self.into(), c);
    }

    fn done(&mut self) {
        self.end_doc()
    }
}

impl<'a> MapTrait for BsonSer<'a> {
    fn field(&mut self, f: &str, s: &dyn Serialize, c: &mut dyn Context) {
        // ? NOTE: We can assume that it will only be used inside this function scope
        self.field = Some(unsafe { std::mem::transmute(f) });
        s.begin(self.into(), c);
        assert!(self.field.is_none(), "field leaked");
    }

    fn done(&mut self) {
        self.end_doc()
    }
}
