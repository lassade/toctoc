use std::borrow::Cow;
use std::collections::{btree_map, hash_map, BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::slice;

use crate::private;
use crate::ser::{Context, Fragment, Map, Seq, Serialize};

impl Context for () {}

impl Serialize for () {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::Null
    }
}

impl Serialize for bool {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::Bool(*self)
    }
}

impl Serialize for str {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

impl Serialize for String {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::Str(Cow::Borrowed(self))
    }
}

macro_rules! unsigned {
    ($ty:ident, $var:ident, $cast:ident) => {
        impl Serialize for $ty {
            fn begin(&self, _c: &dyn Context) -> Fragment {
                Fragment::$var(*self as $cast)
            }
        }
    };
}
unsigned!(u8, U8, u8);
unsigned!(u16, U32, u32);
unsigned!(u32, U32, u32);
unsigned!(u64, U64, u64);
unsigned!(usize, U64, u64);

macro_rules! signed {
    ($ty:ident, $var:ident, $cast:ident) => {
        impl Serialize for $ty {
            fn begin(&self, _c: &dyn Context) -> Fragment {
                Fragment::$var(*self as $cast)
            }
        }
    };
}
signed!(i8, I8, i8);
signed!(i16, I32, i32);
signed!(i32, I32, i32);
signed!(i64, I64, i64);
signed!(isize, I64, i64);

impl Serialize for f32 {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::F32(*self)
    }
}

impl Serialize for f64 {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        Fragment::F64(*self)
    }
}

impl<'a, T: ?Sized + Serialize> Serialize for &'a T {
    fn begin(&self, context: &dyn Context) -> Fragment {
        (**self).begin(context)
    }
}

impl<T: ?Sized + Serialize> Serialize for Box<T> {
    fn begin(&self, context: &dyn Context) -> Fragment {
        (**self).begin(context)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn begin(&self, context: &dyn Context) -> Fragment {
        match self {
            Some(some) => some.begin(context),
            None => Fragment::Null,
        }
    }
}

impl<'a, T: ?Sized + ToOwned + Serialize> Serialize for Cow<'a, T> {
    fn begin(&self, context: &dyn Context) -> Fragment {
        (**self).begin(context)
    }
}

impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        struct TupleStream<'a> {
            first: &'a dyn Serialize,
            second: &'a dyn Serialize,
            state: usize,
        }

        impl<'a> Seq for TupleStream<'a> {
            fn next(&mut self) -> Option<&dyn Serialize> {
                let state = self.state;
                self.state += 1;
                match state {
                    0 => Some(self.first),
                    1 => Some(self.second),
                    _ => None,
                }
            }
        }

        Fragment::Seq(Box::new(TupleStream {
            first: &self.0,
            second: &self.1,
            state: 0,
        }))
    }
}

impl<T: Serialize> Serialize for [T] {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        private::stream_slice(self)
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        private::stream_slice(self)
    }
}

impl<K, V, H> Serialize for HashMap<K, V, H>
where
    K: Hash + Eq + ToString,
    V: Serialize,
    H: BuildHasher,
{
    fn begin(&self, _c: &dyn Context) -> Fragment {
        struct HashMapStream<'a, K: 'a, V: 'a>(hash_map::Iter<'a, K, V>);

        impl<'a, K: ToString, V: Serialize> Map for HashMapStream<'a, K, V> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Owned(k.to_string()), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(HashMapStream(self.iter())))
    }
}

impl<K: ToString, V: Serialize> Serialize for BTreeMap<K, V> {
    fn begin(&self, _c: &dyn Context) -> Fragment {
        private::stream_btree_map(self)
    }
}

impl private {
    pub fn stream_slice<T: Serialize>(slice: &[T]) -> Fragment {
        struct SliceStream<'a, T: 'a>(slice::Iter<'a, T>);

        impl<'a, T: Serialize> Seq for SliceStream<'a, T> {
            fn next(&mut self) -> Option<&dyn Serialize> {
                let element = self.0.next()?;
                Some(element)
            }
        }

        Fragment::Seq(Box::new(SliceStream(slice.iter())))
    }

    pub fn stream_btree_map<K: ToString, V: Serialize>(map: &BTreeMap<K, V>) -> Fragment {
        struct BTreeMapStream<'a, K: 'a, V: 'a>(btree_map::Iter<'a, K, V>);

        impl<'a, K: ToString, V: Serialize> Map for BTreeMapStream<'a, K, V> {
            fn next(&mut self) -> Option<(Cow<str>, &dyn Serialize)> {
                let (k, v) = self.0.next()?;
                Some((Cow::Owned(k.to_string()), v as &dyn Serialize))
            }
        }

        Fragment::Map(Box::new(BTreeMapStream(map.iter())))
    }
}
