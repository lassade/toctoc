use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr;

use crate::json::{drop, Value};

/// A `Vec<Value>` with a non-recursive drop impl.
#[derive(Clone, Debug, Default)]
pub struct Array<'i> {
    inner: Vec<Value<'i>>,
}

impl<'i> Drop for Array<'i> {
    fn drop(&mut self) {
        self.inner.drain(..).for_each(drop::safely);
    }
}

fn take(array: Array) -> Vec<Value> {
    let array = ManuallyDrop::new(array);
    unsafe { ptr::read(&array.inner) }
}

impl<'i> Array<'i> {
    pub fn new() -> Self {
        Array { inner: Vec::new() }
    }
}

impl<'i> Deref for Array<'i> {
    type Target = Vec<Value<'i>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'i> DerefMut for Array<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'i> IntoIterator for Array<'i> {
    type Item = Value<'i>;
    type IntoIter = <Vec<Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        take(self).into_iter()
    }
}

impl<'a, 'i> IntoIterator for &'a Array<'i> {
    type Item = &'a Value<'i>;
    type IntoIter = <&'a Vec<Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'i> IntoIterator for &'a mut Array<'i> {
    type Item = &'a mut Value<'i>;
    type IntoIter = <&'a mut Vec<Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'i> FromIterator<Value<'i>> for Array<'i> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value<'i>>,
    {
        Array {
            inner: Vec::from_iter(iter),
        }
    }
}
