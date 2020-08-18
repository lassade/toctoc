use crate::de::{Context, Deserialize};
use crate::owned::OwnedRaw;
use crate::Result;
use std::mem::transmute;
use std::pin::Pin;

pub type Owned<T> = OwnedRaw<Vec<u8>, T>;

pub fn from_bin_owned<'de, T: Deserialize<'de>>(
    mut data: Vec<u8>,
    ctx: &mut dyn Context,
) -> Result<Owned<T>> {
    unsafe {
        let inner: T = super::from_bin(transmute(data.as_mut_slice()), ctx)?;
        Ok(Owned {
            data: Pin::new(data),
            value: Some(inner),
        })
    }
}
