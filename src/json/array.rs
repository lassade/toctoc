use std::iter::FromIterator;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr;

use crate::json::{drop, Value};

/// A `Vec<Value>` with a non-recursive drop impl.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Array<'de> {
    inner: Vec<Value<'de>>,
}

impl<'de> Drop for Array<'de> {
    fn drop(&mut self) {
        self.inner.drain(..).for_each(drop::safely);
    }
}

fn take(array: Array) -> Vec<Value> {
    let array = ManuallyDrop::new(array);
    unsafe { ptr::read(&array.inner) }
}

impl<'de> Array<'de> {
    pub fn new() -> Self {
        Array { inner: Vec::new() }
    }
}

impl<'de> Deref for Array<'de> {
    type Target = Vec<Value<'de>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'de> DerefMut for Array<'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'de> IntoIterator for Array<'de> {
    type Item = Value<'de>;
    type IntoIter = <Vec<Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        take(self).into_iter()
    }
}

impl<'a, 'de> IntoIterator for &'a Array<'de> {
    type Item = &'a Value<'de>;
    type IntoIter = <&'a Vec<Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'de> IntoIterator for &'a mut Array<'de> {
    type Item = &'a mut Value<'de>;
    type IntoIter = <&'a mut Vec<Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'de> FromIterator<Value<'de>> for Array<'de> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = Value<'de>>,
    {
        Array {
            inner: Vec::from_iter(iter),
        }
    }
}
