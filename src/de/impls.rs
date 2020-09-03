use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::str::FromStr;

use crate::de::{Context, Deserialize, Map, Seq, Visitor};
use crate::error::{Error, Result};
use crate::Place;

impl<'de> Deserialize<'de> for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de> Visitor<'de> for Place<()> {
            fn null(&mut self, _: &mut dyn Context) -> Result<()> {
                self.out = Some(());
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'de> Deserialize<'de> for bool {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de> Visitor<'de> for Place<bool> {
            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(b);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'de> Deserialize<'de> for String {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de> Visitor<'de> for Place<String> {
            fn string(&mut self, s: &str, _: &mut dyn Context) -> Result<()> {
                self.out = Some(s.to_owned());
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for &'a str {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'a, 'de: 'a> Visitor<'de> for Place<&'a str> {
            fn string(&mut self, s: &'de str, _: &mut dyn Context) -> Result<()> {
                self.out = Some(s);
                Ok(())
            }
        }
        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl<'de> Deserialize<'de> for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
                impl<'de> Visitor<'de> for Place<$ty> {
                    fn negative(&mut self, n: i64, _: &mut dyn Context) -> Result<()> {
                        if n >= $ty::min_value() as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error::out_of_range(stringify!($ty)))?
                        }
                    }

                    fn nonnegative(&mut self, n: u64, _: &mut dyn Context) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error::out_of_range(stringify!($ty)))?
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
signed!(i8);
signed!(i16);
signed!(i32);
signed!(i64);
signed!(isize);

macro_rules! unsigned {
    ($ty:ident) => {
        impl<'de> Deserialize<'de> for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
                impl<'de> Visitor<'de> for Place<$ty> {
                    fn nonnegative(&mut self, n: u64, _: &mut dyn Context) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error::out_of_range(stringify!($ty)))?
                        }
                    }
                }
                Place::new(out)
            }
        }
    };
}
unsigned!(u8);
unsigned!(u16);
unsigned!(u32);
unsigned!(u64);
unsigned!(usize);

// * MOD: Better support for single and double precistion
// * floats (avoid any expensive casts whenever possible)

impl<'de> Deserialize<'de> for f64 {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de> Visitor<'de> for Place<f64> {
            fn negative(&mut self, n: i64, _: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f64);
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f64);
                Ok(())
            }

            fn double(&mut self, n: f64) -> Result<()> {
                self.out = Some(n as f64);
                Ok(())
            }

            fn single(&mut self, n: f32) -> Result<()> {
                self.out = Some(n as f64);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'de> Deserialize<'de> for f32 {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de> Visitor<'de> for Place<f32> {
            fn negative(&mut self, n: i64, _: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f32);
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f32);
                Ok(())
            }

            fn double(&mut self, n: f64) -> Result<()> {
                if n <= (f32::MAX as f64) && n >= (f32::MIN as f64) {
                    self.out = Some(n as f32);
                    Ok(())
                } else {
                    Err(Error::out_of_range("f32"))?
                }
            }

            fn single(&mut self, n: f32) -> Result<()> {
                self.out = Some(n as f32);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Box<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, T> Visitor<'de> for Place<Box<T>>
        where
            T: Deserialize<'de>,
        {
            fn null(&mut self, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).null(c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).boolean(b)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn string(&mut self, s: &'de str, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).string(s, c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn negative(&mut self, n: i64, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).negative(n, c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).nonnegative(n, c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn single(&mut self, n: f32) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).single(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn double(&mut self, n: f64) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).double(n)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).seq(s, c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).map(m, c)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, T> Visitor<'de> for Place<Option<T>>
        where
            T: Deserialize<'de>,
        {
            fn null(&mut self, _: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &'de str, c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).string(s, c)
            }

            fn negative(&mut self, n: i64, c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).negative(n, c)
            }

            fn nonnegative(&mut self, n: u64, c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).nonnegative(n, c)
            }

            fn single(&mut self, n: f32) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).single(n)
            }

            fn double(&mut self, n: f64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).double(n)
            }

            fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).seq(s, c)
            }

            fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).map(m, c)
            }
        }

        Place::new(out)
    }
}

macro_rules! tuple {
    ($(<$($n:ident $i:literal),*>),*) => { $(
        impl<'de, $($n: Deserialize<'de>,)*> Deserialize<'de> for ($($n,)*) {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
                impl<'de, $($n: Deserialize<'de>,)*> Visitor<'de> for Place<($($n,)*)> {
                    #[allow(non_snake_case)]
                    fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
                        self.out = Some((
                        $({
                            let mut value: Option<$n> = None;
                            s.visit(Deserialize::begin(&mut value), c)?;
                            value.ok_or(Error::missing_element($i))?
                        },)*
                        ));
                        while s.visit(Visitor::ignore(), c)? {}
                        Ok(())
                    }
                }

                Place::new(out)
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

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, T: Deserialize<'de>> Visitor<'de> for Place<Vec<T>> {
            fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
                let mut vec = vec![];
                let mut element = None;
                while s.visit(Deserialize::begin(&mut element), c)? {
                    element.take().map(|e| vec.push(e));
                }
                self.out = Some(vec);
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'de, K, V, H> Deserialize<'de> for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Deserialize<'de>,
    H: BuildHasher + Default,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, K, V, H> Visitor<'de> for Place<HashMap<K, V, H>>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize<'de>,
            H: BuildHasher + Default,
        {
            fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
                let mut hashmap = HashMap::with_hasher(H::default());
                let mut element = None;
                while let Some(k) = m.next()? {
                    let k = K::from_str(k).map_err(|_| Error::invalid_map_key(k.to_string()))?;
                    m.visit(Deserialize::begin(&mut element), c)?;
                    element.take().map(|e| hashmap.insert(k, e));
                }
                self.out = Some(hashmap);
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'de, K: FromStr + Ord, V: Deserialize<'de>> Deserialize<'de> for BTreeMap<K, V> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, K: FromStr + Ord, V: Deserialize<'de>> Visitor<'de> for Place<BTreeMap<K, V>> {
            fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
                let mut btree = BTreeMap::new();
                let mut element = None;
                while let Some(k) = m.next()? {
                    let k = K::from_str(k).map_err(|_| Error::invalid_map_key(k.to_string()))?;
                    m.visit(Deserialize::begin(&mut element), c)?;
                    element.take().map(|e| btree.insert(k, e));
                }
                self.out = Some(btree);
                Ok(())
            }
        }

        Place::new(out)
    }
}
