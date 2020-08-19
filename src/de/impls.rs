use paste::paste;
use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::mem;
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
                            Err(Error)?
                        }
                    }

                    fn nonnegative(&mut self, n: u64, _: &mut dyn Context) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)?
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
                            Err(Error)?
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
                    Err(Error)?
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

// macro_rules! de_tuple {
//     ($($tt:ident),*) => {
//         impl<'de, $($tt: Deserialize<'de>,)*> Deserialize<'de> for ($($tt,)*) {
//             fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//                 impl<'de, $($tt: Deserialize<'de>,)*> Visitor<'de> for Place<($($tt,)*)> {
//                     #[allow(non_snake_case)]
//                     fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
//                         $(paste! {
//                             let mut [<v $tt>] = None;
//                             s.visit(Deserialize::begin(&mut [<v $tt>]), c)?;
//                         })*
//                         self.out = Some((
//                             $( paste! { [<v $tt>].ok_or(Error)? },)*
//                         ));
//                         while s.visit(Visitor::ignore(), c)? {}
//                         Ok(())
//                     }
//                 }

//                 Place::new(out)
//             }
//         }
//     };
// }

//de_tuple!(T0, T1);

impl<'de, A: Deserialize<'de>, B: Deserialize<'de>> Deserialize<'de> for (A, B) {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
        impl<'de, A: Deserialize<'de>, B: Deserialize<'de>> Visitor<'de> for Place<(A, B)> {
            fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
                let mut a = None;
                s.visit(Deserialize::begin(&mut a), c)?;
                let mut b = None;
                s.visit(Deserialize::begin(&mut b), c)?;
                self.out = Some((a.take().ok_or(Error)?, b.take().ok_or(Error)?));
                while s.visit(Visitor::ignore(), c)? {}
                Ok(())
            }
        }

        Place::new(out)
    }
}

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

// impl<'de, K, V, H> Deserialize<'de> for HashMap<K, V, H>
// where
//     K: FromStr + Hash + Eq,
//     V: Deserialize<'de>,
//     H: BuildHasher + Default,
// {
//     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//         impl<'de, K, V, H> Visitor<'de> for Place<HashMap<K, V, H>>
//         where
//             K: FromStr + Hash + Eq,
//             V: Deserialize<'de>,
//             H: BuildHasher + Default,
//         {
//             fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'de> + 'a>>
//             where
//                 'de: 'a,
//             {
//                 Ok(Box::new(MapBuilder {
//                     out: &mut self.out,
//                     map: HashMap::with_hasher(H::default()),
//                     key: None,
//                     value: None,
//                 }))
//             }
//         }

//         struct MapBuilder<'a, K: 'a, V: 'a, H: 'a> {
//             out: &'a mut Option<HashMap<K, V, H>>,
//             map: HashMap<K, V, H>,
//             key: Option<K>,
//             value: Option<V>,
//         }

//         impl<'a, K: Hash + Eq, V, H: BuildHasher> MapBuilder<'a, K, V, H> {
//             fn shift(&mut self) {
//                 if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
//                     self.map.insert(k, v);
//                 }
//             }
//         }

//         impl<'de, 'a, K, V, H> Map<'de> for MapBuilder<'a, K, V, H>
//         where
//             K: FromStr + Hash + Eq,
//             V: Deserialize<'de>,
//             H: BuildHasher + Default,
//         {
//             fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'de>> {
//                 self.shift();
//                 self.key = Some(match K::from_str(k) {
//                     Ok(key) => key,
//                     Err(_) => return Err(Error),
//                 });
//                 Ok(Deserialize::begin(&mut self.value))
//             }

//             fn finish(&mut self, _: &mut dyn Context) -> Result<()> {
//                 self.shift();
//                 let substitute = HashMap::with_hasher(H::default());
//                 *self.out = Some(mem::replace(&mut self.map, substitute));
//                 Ok(())
//             }
//         }

//         Place::new(out)
//     }
// }

// impl<'de, K: FromStr + Ord, V: Deserialize<'de>> Deserialize<'de> for BTreeMap<K, V> {
//     fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'de> {
//         impl<'de, K: FromStr + Ord, V: Deserialize<'de>> Visitor<'de> for Place<BTreeMap<K, V>> {
//             fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'de> + 'a>>
//             where
//                 'de: 'a,
//             {
//                 Ok(Box::new(MapBuilder {
//                     out: &mut self.out,
//                     map: BTreeMap::new(),
//                     key: None,
//                     value: None,
//                 }))
//             }
//         }

//         struct MapBuilder<'a, K: 'a, V: 'a> {
//             out: &'a mut Option<BTreeMap<K, V>>,
//             map: BTreeMap<K, V>,
//             key: Option<K>,
//             value: Option<V>,
//         }

//         impl<'a, K: Ord, V> MapBuilder<'a, K, V> {
//             fn shift(&mut self) {
//                 if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
//                     self.map.insert(k, v);
//                 }
//             }
//         }

//         impl<'de, 'a, K: FromStr + Ord, V: Deserialize<'de>> Map<'de> for MapBuilder<'a, K, V> {
//             fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'de>> {
//                 self.shift();
//                 self.key = Some(match K::from_str(k) {
//                     Ok(key) => key,
//                     Err(_) => return Err(Error),
//                 });
//                 Ok(Deserialize::begin(&mut self.value))
//             }

//             fn finish(&mut self, _: &mut dyn Context) -> Result<()> {
//                 self.shift();
//                 *self.out = Some(mem::replace(&mut self.map, BTreeMap::new()));
//                 Ok(())
//             }
//         }

//         Place::new(out)
//     }
// }
