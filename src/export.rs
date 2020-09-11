pub use std::borrow::Cow;
pub use std::boxed::Box;
pub use std::option::Option::{self, None, Some};
pub use std::result::Result::{Err, Ok};
pub use std::string::String;

pub use self::help::Str as str;
pub use self::help::Usize as usize;

mod help {
    pub type Str = str;
    pub type Usize = usize;
}

/// Context hint to help decide which `Entity` or `Asset` return
pub enum Hint<'a> {
    /// Null or default asset or entity
    Null,
    /// Stable index number
    Number(u64),
    /// May contain: path, name, formatted guid or inlined json data
    Str(&'a str),
    /// May contain: guid or inlined bson data
    Bytes(&'a [u8]),
}

/// Implementation independent asset handle
pub enum AssetHandle<T> {
    Atomic(std::sync::Arc<T>),
    RefCounted(std::rc::Rc<T>),
    Plain(T),
}

/// Asset handle with type information
pub struct Asset {
    pub handle: AssetHandle<u32>,
    pub id: (std::any::TypeId, u32),
}

/// Entity type, should be compatible with most ecs crates
pub struct Entity(pub u64);

/// Hex conversion utility
pub use bintext::hex;
