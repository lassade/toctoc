
use std::pin::Pin;
use std::mem::transmute;
use crate::de::{Deserialize, Context};
use crate::owned::OwnedRaw;
use crate::Result;

pub type Owned<T> = OwnedRaw<Vec<u8>, T>;

pub fn from_bin_owned<'de, T: Deserialize<'de>>(mut data: Vec<u8>, ctx: &mut dyn Context) -> Result<Owned<T>> {
    unsafe { 
        let inner: T = super::from_bin(transmute(data.as_mut_slice()), ctx)?;
        Ok(Owned {
            data: Pin::new(data),
            value: Some(inner),
        })
    }
}
