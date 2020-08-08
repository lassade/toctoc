use std::pin::Pin;

/// Keeps a value `T` that contains borrows to some data `D`.
/// Meant for moving zero copy structs and theis data around.
///
/// Most recomended way is to use as a read only primitive,
/// but also implements `as_mut` for mutability.
#[allow(dead_code)]
pub struct OwnedRaw<D, T> {
    pub(crate) data: Pin<D>,
    pub(crate) value: Option<T>,
}

impl<D, T> OwnedRaw<D, T> {
    /// Gets a reference of `T`.
    ///
    /// ***Warning*** Rust doesn't support self borrowing thus is
    /// un anble to savely handle this reference. Calling `clone`
    /// or `to_owned` on this reference will detach it from the underlaing
    /// data that can be droped before the cloned ref, resulting in a 
    /// use after free error.
    pub unsafe fn as_ref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
    
    /// Gets a mutable reference of `T`.
    ///
    /// ***Warning*** Rust doesn't support self borrowing thus is
    /// un anble to savely handle this reference. Calling `clone`
    /// or `to_owned` on this reference will detach it from the underlaing
    /// data that can be droped before the cloned ref, resulting in a 
    /// use after free error.
    pub unsafe fn as_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}

impl<D, T> std::ops::Drop for OwnedRaw<D, T> {
    fn drop(&mut self) {
        self.value = None; // Drop the inner borrowed value frist
    }
}