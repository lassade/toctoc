use std::mem::{align_of, size_of};
use crate::{ser, de, Result, Error, Place};
use crate::export::Cow;

/// Wrapper around slices or vec to be (de)serialize as bytes
#[derive(Default, Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Bytes<T: Binary>(pub T);

impl<T: Binary> ser::Serialize for Bytes<T> {
    fn begin(&self, _: &dyn ser::Context) -> ser::Fragment {
        let (b, align) =  Binary::as_bytes(&self.0);
        ser::Fragment::Bin {
            bytes: Cow::Borrowed(b),
            align, 
        }
    }
}

impl<'de, T: Binary> de::Deserialize<'de> for Bytes<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor<'de> {
        impl<'de, T1: Binary> de::Visitor<'de> for Place<Bytes<T1>> {
            fn bytes(&mut self, b: &'de [u8], _: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bytes(T1::from_bytes(b)?));
                Ok(())
            }
        }
        Place::new(out)
    }
}

/// Implemented by any type that can be converted into or from bytes
pub trait Binary: Sized {
    /// Returns a byte slice and alignment for this binary type
    fn as_bytes(&self) -> (&[u8], usize);
    /// Makes a new `Self` from bytes.
    /// ***NOTE*** This function should to check memory alignment first
    fn from_bytes(bytes: &[u8]) -> Result<Self>;
}

impl<T: ByValue> Binary for Vec<T> {
    fn as_bytes(&self) -> (&[u8], usize) {
        (
            unsafe { 
                std::slice::from_raw_parts(
                    self.as_ptr() as *const u8,
                    self.len() * size_of::<T>()
                )
            },
            align_of::<T>()
        )
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let slice = <&[T]>::from_bytes(bytes)?;
        let mut vec = Vec::with_capacity(slice.len());
        vec.extend_from_slice(slice);
        Ok(vec)
    }
}

impl<T: ByValue> Binary for &[T] {
    fn as_bytes(&self) -> (&[u8], usize) {
        (
            unsafe { 
                std::slice::from_raw_parts(
                    self.as_ptr() as *const u8,
                    self.len() * size_of::<T>()
                )
            },
            align_of::<T>()
        )
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.as_ptr().align_offset(align_of::<T>()) != 0 {
            Err(Error)?
        }

        unsafe {
            Ok(std::slice::from_raw_parts(
                bytes.as_ptr() as *const T, 
                bytes.len() / size_of::<T>()
            ))
        }
    }
}

/// Blanket trait implemented by all types that are represented by value
pub unsafe trait ByValue: Copy {}

macro_rules! by_val {
    ($($t:tt),*) => { $(unsafe impl ByValue for $t {})* };
    (<$($v:literal),*>) => {
        $(unsafe impl<T:ByValue> ByValue for [T; $v] {})*
    };
    ($(>$($t:ident),*<),*) => {
        $(unsafe impl<$($t: ByValue,)*> ByValue for ($($t),*) {})*
    };
}

by_val!(char, u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, f32, f64);
by_val!(< 1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16,
         17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32>);
by_val!(>T0, T1<,
        >T1, T2, T3<,
        >T1, T2, T3, T4<,
        >T1, T2, T3, T4, T5<,
        >T1, T2, T3, T4, T5, T6<,
        >T1, T2, T3, T4, T5, T6, T7<,
        >T1, T2, T3, T4, T5, T6, T7, T8<);


///////////////////////////////////////////////////////////////////////////////


/// Best memory alignment guess.
///
/// Figure out the highest rank alignment of a pointer. 
/// A higher the alignment rank have more memory flexibility, which means
/// it can be casted to any type that require a lower rank alignment.
pub fn guess_align_of<T>(p: *const T) -> usize {
    const ALIGNMENTS: [usize; 6] = [1, 2, 4, 8, 16, 32];
    let p = p as usize;
    1 << ALIGNMENTS[..].iter()
            .position(|a| (*a & p) != 0)
            .unwrap_or(6)
}


///////////////////////////////////////////////////////////////////////////////


#[cfg(test)]
mod tests {
    use std::mem::align_of;
    use super::*;

    macro_rules! is_algin {
        ($($t:tt),*) => { $({
            let v: $t = 0;
            let p = &v as *const _;
            let g = guess_align_of(p);
            let e = align_of::<$t>();
            assert!(e <= g, "{} <= {} => {:?}", e, g, p);
        })* };
    }

    #[test]
    fn align_guessing() {
        is_algin!(u8, u16, u32, u64, u128);
    }

    #[test]
    fn binary_cast() {
        let v = vec![(4u32, 4u32), (4u32, 4u32), (4u32, 4u32)];
        let (bytes, a) = v.as_bytes();
        assert_eq!(a, align_of::<(u32, u32)>());

        let s = <&[(u32, u32)]>::from_bytes(bytes).unwrap();
        assert_eq!(s, &v[..]);

        let a = <Vec<(u32, u32)>>::from_bytes(bytes).unwrap();
        assert_eq!(a, v);
    }
}