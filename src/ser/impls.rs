use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};

use crate::ser::{Context, Done, Serialize, Visitor};

impl Serialize for () {
    fn begin(&self, v: Visitor, _: &mut dyn Context) -> Done {
        v.null()
    }
}

impl Serialize for bool {
    fn begin(&self, v: Visitor, _: &mut dyn Context) -> Done {
        v.boolean(*self)
    }
}

impl Serialize for str {
    fn begin(&self, v: Visitor, _: &mut dyn Context) -> Done {
        v.string(self)
    }
}

impl Serialize for String {
    fn begin(&self, v: Visitor, _: &mut dyn Context) -> Done {
        v.string(self)
    }
}

macro_rules! primitive {
    ($ty:ident, $method:ident, $cast:ident) => {
        impl Serialize for $ty {
            fn begin(&self, v: Visitor, _: &mut dyn Context) -> Done {
                v.$method(*self as $cast)
            }
        }
    };
}

primitive!(u8, byte, u8);
primitive!(u16, uint, u32);
primitive!(u32, uint, u32);
primitive!(u64, ulong, u64);
primitive!(usize, ulong, u64);
primitive!(i8, sbyte, i8);
primitive!(i16, int, i32);
primitive!(i32, int, i32);
primitive!(i64, long, i64);
primitive!(isize, long, i64);
primitive!(f32, single, f32);
primitive!(f64, double, f64);

impl<'a, T: ?Sized + Serialize> Serialize for &'a T {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        (**self).begin(v, context)
    }
}

impl<T: ?Sized + Serialize> Serialize for Box<T> {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        (**self).begin(v, context)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        match self {
            Some(some) => some.begin(v, context),
            None => v.null(),
        }
    }
}

impl<'a, T: ?Sized + ToOwned + Serialize> Serialize for Cow<'a, T> {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        (**self).begin(v, context)
    }
}

macro_rules! arrays {
    ($($n:tt),*) => { $(
        impl<T: Serialize> Serialize for [T; $n] {
            fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
                self[..].begin(v, context)
            }
        }
    )*
    };
}

arrays!(
    2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27,
    28, 29, 30, 31, 32
);

impl<A: Serialize> Serialize for (A,) {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        self.0.begin(v, context)
    }
}

macro_rules! tuple {
    ($(<$($n:ident $i:tt),*>),*) => { $(
        impl<$($n: Serialize,)*> Serialize for ($($n,)*) {
            fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
                v.seq()
                $(.element(&self.$i, context))*
                .done()
            }
        }
    )*
    };
}

tuple!(
    <A 0, B 1>,
    <A 0, B 1, C 2>,
    <A 0, B 1, C 2, D 3>,
    <A 0, B 1, C 2, D 3, E 4>,
    <A 0, B 1, C 2, D 3, E 4, F 5>,
    <A 0, B 1, C 2, D 3, E 4, F 5, G 6>,
    <A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7>
);

impl<T: Serialize> Serialize for [T] {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        let mut seq = v.seq();
        for e in self {
            seq = seq.element(e, context);
        }
        seq.done()
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        self.as_slice().begin(v, context)
    }
}

impl<K, V, H> Serialize for HashMap<K, V, H>
where
    K: Hash + Eq + ToString,
    V: Serialize,
    H: BuildHasher,
{
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        let mut map = v.map();
        for (k, e) in self {
            map = map.field(&k.to_string(), e, context);
        }
        map.done()
    }
}

impl<K: ToString, V: Serialize> Serialize for BTreeMap<K, V> {
    fn begin(&self, v: Visitor, context: &mut dyn Context) -> Done {
        let mut map = v.map();
        for (k, e) in self {
            map = map.field(&k.to_string(), e, context);
        }
        map.done()
    }
}
