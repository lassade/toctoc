//! Memory aligned buffer to help with binary serialization

use paste::paste;
use std::alloc::{alloc, dealloc, realloc, Layout};
use std::ptr::null_mut;
use std::slice::IterMut;

/// Like a byte `Vec` but with underling buffer always aligned with
/// the maximum alignment requirement given by `extend_from_slice_aligned`
pub struct Buffer {
    ptr: *mut u8,
    ptr_align: usize,
    cap: usize,
    len: usize,
}

impl Buffer {
    /// Buffer start alignment
    pub const ALIGNMENT: usize = 4;

    pub fn new() -> Self {
        Buffer {
            ptr: null_mut(),
            ptr_align: Self::ALIGNMENT,
            cap: 0,
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.ptr
    }

    pub fn as_slice(&self) -> &[u8] {
        if self.cap == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
        }
    }

    pub fn reserve(&mut self, len: usize) {
        if len > self.cap {
            let cap = (len << 1).max(4); // new capacity
            let layout = Layout::from_size_align(cap, self.ptr_align).unwrap();

            let ptr = unsafe {
                if self.cap == 0 {
                    alloc(layout)
                } else {
                    realloc(
                        self.ptr,
                        Layout::from_size_align_unchecked(self.len, self.ptr_align),
                        cap,
                    )
                }
            };

            self.ptr = ptr;
            self.cap = cap;
        }
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        let len = slice.len() + self.len;
        self.reserve(len);

        unsafe {
            std::ptr::copy_nonoverlapping(slice.as_ptr(), self.ptr.add(self.len), slice.len());
        }

        self.len = len;
    }

    /// Extends the buffer repeating the same byte `val` a
    /// certain amount of `times`
    pub fn extend_repeating(&mut self, val: u8, times: usize) {
        self.reserve(self.len + times);
        for i in 0..times {
            unsafe {
                *self.ptr.add(self.len + i) = val;
            }
        }
        self.len += times;
    }

    /// Extends the buffer but keeps the `slice` aligned with `align`,
    /// to ensure alignment this function will add a padding before the data.
    ///
    /// Returns the index on the buffer where the written `slice` starts
    pub fn extend_from_slice_aligned(&mut self, slice: &[u8], align: usize) -> usize {
        let padding = unsafe { self.ptr.add(self.len).align_offset(align) };
        self.ptr_align = self.ptr_align.max(align); // Alloc using the maximum allocation
        self.extend_repeating(0, padding);
        let start = self.len();
        self.extend_from_slice(slice);
        start
    }

    pub fn to_vec(self) -> Vec<u8> {
        unsafe {
            let vec = Vec::from_raw_parts(self.ptr, self.len, self.cap);
            std::mem::forget(self); // Avoid double free
            vec
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, u8> {
        if self.cap == 0 {
            [].iter_mut()
        } else {
            unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len).iter_mut() }
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, index: usize) -> &u8 {
        &*self.ptr.add(index)
    }

    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self, index: usize) -> &mut u8 {
        &mut *self.ptr.add(index)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            if self.ptr != null_mut() {
                dealloc(
                    self.ptr,
                    Layout::from_size_align_unchecked(self.cap, Self::ALIGNMENT),
                )
            }
        }
    }
}

macro_rules! write_impl {
    ($($t:ty),*) => {
        impl Buffer {
            $(paste! {
                pub fn [<write_ $t>] (&mut self, value: $t) {
                    self.extend_from_slice(&value.to_le_bytes()[..]);
                }
            })*
        }
    };
}

write_impl!(u8, i8, u32, i32, u64, i64, f32, f64);

macro_rules! replace_impl {
    ($($t:ty),*) => {
        impl Buffer {
            $(paste! {
                pub fn [<replace_ $t>] (&mut self, index: usize, value: $t) {
                    self.iter_mut()
                        .skip(index)
                        .zip(value.to_le_bytes().iter().copied())
                        .for_each(|(b, a)| *b = a);
                }
            })*
        }
    };
}

// ? NOTE: This is the only needed value
replace_impl!(u32);

impl std::ops::Index<usize> for Buffer {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        if index >= self.len {
            panic!()
        }
        unsafe { self.get_unchecked(index) }
    }
}

impl std::ops::IndexMut<usize> for Buffer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if index >= self.len {
            panic!()
        }
        unsafe { self.get_mut_unchecked(index) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytes::Binary;

    #[test]
    fn write_data() {
        let mut buf = Buffer::new();
        assert_eq!(buf.len(), 0);

        let d = &[0, 1, 2, 3, 4, 5][..];
        buf.extend_from_slice(d);
        assert_eq!(buf.len(), 6);
        assert_eq!(buf.ptr.align_offset(4), 0); // Alignment was kept

        let v = buf.to_vec();
        assert_eq!(v, d.to_vec());
    }

    #[test]
    fn write_data_aligned() {
        let mut buf = Buffer::new();
        let v = &[[4u32, 4u32], [4u32, 4u32], [4u32, 4u32]][..];
        let (b, a) = v.as_bytes();
        buf.write_u8(1);
        let i = buf.extend_from_slice_aligned(b, a);
        assert_eq!(unsafe { buf.as_ptr().add(i) }.align_offset(a), 0);

        let r = <&[[u32; 2]]>::from_bytes(&buf.as_slice()[i..]).unwrap();
        assert!(r.iter().eq(v.iter()));
    }

    #[test]
    fn write_data_with_mutiple_alignments() {
        let mut buf = Buffer::new();

        // ? NOTE: odd number of bytes on purpose
        let v = &b"\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\
                          \xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\
                          \xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\
                          \xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\xa5\
                          \xa5\xa5\xa5\xa5\xa5"[..];

        let a = buf.extend_from_slice_aligned(v, 2);
        let b = buf.extend_from_slice_aligned(v, 8);
        let c = buf.extend_from_slice_aligned(v, 16);
        let d = buf.extend_from_slice_aligned(v, 1);

        // Check alignment and equality
        assert_eq!(unsafe { buf.as_ptr().add(a) }.align_offset(2), 0);
        assert!(buf.as_slice().iter().skip(a).take(v.len()).eq(v.iter()));

        assert_eq!(unsafe { buf.as_ptr().add(b) }.align_offset(8), 0);
        assert!(buf.as_slice().iter().skip(b).take(v.len()).eq(v.iter()));

        assert_eq!(unsafe { buf.as_ptr().add(c) }.align_offset(16), 0);
        assert!(buf.as_slice().iter().skip(c).take(v.len()).eq(v.iter()));

        assert_eq!(unsafe { buf.as_ptr().add(d) }.align_offset(1), 0);
        assert!(buf.as_slice().iter().skip(d).take(v.len()).eq(v.iter()));
    }
}
