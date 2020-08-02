//! BSON data format.
//! 
//! Both serialzier and deserializer warps the data in a root level
//! document with a single field with an empty name. For instance 
//! `true` is serialized as:
//!
//! ```text
//! \x08000000
//! \x08 \x00 \x01
//! \x00
//! ```


mod ser;
pub use self::ser::*;

mod de;
pub use self::de::*;

//mod primitive;
//pub use self::primitive::*;