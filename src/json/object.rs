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
pub struct Object<'i> {
    inner: BTreeMap<String, Value<'i>>,
}

impl<'i> Drop for Object<'i> {
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

impl<'i> Object<'i> {
    pub fn new() -> Self {
        Object {
            inner: BTreeMap::new(),
        }
    }
}

impl<'i> Deref for Object<'i> {
    type Target = BTreeMap<String, Value<'i>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'i> DerefMut for Object<'i> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'i> IntoIterator for Object<'i> {
    type Item = (String, Value<'i>);
    type IntoIter = <BTreeMap<String, Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        take(self).into_iter()
    }
}

impl<'a, 'i: 'a> IntoIterator for &'a Object<'i> {
    type Item = (&'a String, &'a Value<'i>);
    type IntoIter = <&'a BTreeMap<String, Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'i: 'a> IntoIterator for &'a mut Object<'i> {
    type Item = (&'a String, &'a mut Value<'i>);
    type IntoIter = <&'a mut BTreeMap<String, Value<'i>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'i> FromIterator<(String, Value<'i>)> for Object<'i> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (String, Value<'i>)>,
    {
        Object {
            inner: BTreeMap::from_iter(iter),
        }
    }
}

impl private {
    pub fn stream_object<'o, 'i: 'o>(object: &'o Object<'i>) -> Fragment<'o> {
        struct ObjectIter<'o, 'i: 'o>(btree_map::Iter<'o, String, Value<'i>>);

        impl<'o, 'i: 'o> ser::Map for ObjectIter<'o, 'i> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Borrowed(k), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(ObjectIter(object.iter())))
    }
}
