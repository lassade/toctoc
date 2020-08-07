use std::borrow::Cow;
use std::collections::{btree_map, BTreeMap};
use std::iter::FromIterator;
use std::mem::{self, ManuallyDrop};
use std::ops::{Deref, DerefMut};
use std::ptr;

use crate::json::{drop, Value};
use crate::private;
use crate::ser::{self, Fragment, Serialize};

/// A `BTreeMap<String, Value>` with a non-recursive drop impl.
#[derive(Clone, Debug, Default)]
pub struct Object<'de> {
    inner: BTreeMap<String, Value<'de>>,
}

impl<'de> Drop for Object<'de> {
    fn drop(&mut self) {
        for (_, child) in mem::replace(&mut self.inner, BTreeMap::new()) {
            drop::safely(child);
        }
    }
}

fn take(object: Object) -> BTreeMap<String, Value> {
    let object = ManuallyDrop::new(object);
    unsafe { ptr::read(&object.inner) }
}

impl<'de> Object<'de> {
    pub fn new() -> Self {
        Object {
            inner: BTreeMap::new(),
        }
    }
}

impl<'de> Deref for Object<'de> {
    type Target = BTreeMap<String, Value<'de>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'de> DerefMut for Object<'de> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'de> IntoIterator for Object<'de> {
    type Item = (String, Value<'de>);
    type IntoIter = <BTreeMap<String, Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        take(self).into_iter()
    }
}

impl<'a, 'de: 'a> IntoIterator for &'a Object<'de> {
    type Item = (&'a String, &'a Value<'de>);
    type IntoIter = <&'a BTreeMap<String, Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'de: 'a> IntoIterator for &'a mut Object<'de> {
    type Item = (&'a String, &'a mut Value<'de>);
    type IntoIter = <&'a mut BTreeMap<String, Value<'de>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'de> FromIterator<(String, Value<'de>)> for Object<'de> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Value<'de>)>,
    {
        Object {
            inner: BTreeMap::from_iter(iter),
        }
    }
}

impl private {
    pub fn stream_object<'o, 'de: 'o>(object: &'o Object<'de>) -> Fragment<'o> {
        struct ObjectIter<'o, 'de: 'o>(btree_map::Iter<'o, String, Value<'de>>);

        impl<'o, 'de: 'o> ser::Map for ObjectIter<'o, 'de> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Borrowed(k), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(ObjectIter(object.iter())))
    }
}
