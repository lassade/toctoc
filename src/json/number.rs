/// A JSON number represented by some Rust primitive.
#[derive(Clone, Debug, PartialEq)]
pub enum Number {
    U64(u64),
    I64(i64),
    F32(f32), // * MOD: Single precision to avoid casting between f32 and f64
    F64(f64),
}
