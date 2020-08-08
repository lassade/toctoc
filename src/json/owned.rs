
use std::pin::Pin;
use std::mem::transmute;
use crate::de::{Deserialize, Context};
use crate::owned::OwnedRaw;
use crate::Result;

pub type Owned<T> = OwnedRaw<String, T>;

pub fn from_str_owned<'de, T: Deserialize<'de>>(mut data: String, ctx: &mut dyn Context) -> Result<Owned<T>> {
    unsafe { 
        let inner: T = super::from_str(transmute(data.as_mut_str()), ctx)?;
        Ok(Owned {
            data: Pin::new(data),
            value: Some(inner),
        })
    }
}
