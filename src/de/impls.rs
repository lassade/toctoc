use std::collections::{BTreeMap, HashMap};
use std::hash::{BuildHasher, Hash};
use std::mem;
use std::str::FromStr;

use crate::de::{Deserialize, Map, Seq, Visitor, Context};
use crate::error::{Error, Result};
use crate::Place;

impl Context for () {}

impl<'i> Deserialize<'i> for () {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<()> {
            fn null(&mut self, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(());
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'i> Deserialize<'i> for bool {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<bool> {
            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(b);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<'i> Deserialize<'i> for String {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<String> {
            fn string(&mut self, s: &str, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(s.to_owned());
                Ok(())
            }
        }
        Place::new(out)
    }
}

macro_rules! signed {
    ($ty:ident) => {
        impl<'i> Deserialize<'i> for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
                impl<'i> Visitor<'i> for Place<$ty> {
                    fn negative(&mut self, n: i64, _c: &mut dyn Context) -> Result<()> {
                        if n >= $ty::min_value() as i64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
                        }
                    }

                    fn nonnegative(&mut self, n: u64, _c: &mut dyn Context) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
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
        impl<'i> Deserialize<'i> for $ty {
            fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
                impl<'i> Visitor<'i> for Place<$ty> {
                    fn nonnegative(&mut self, n: u64, _c: &mut dyn Context) -> Result<()> {
                        if n <= $ty::max_value() as u64 {
                            self.out = Some(n as $ty);
                            Ok(())
                        } else {
                            Err(Error)
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

impl<'i> Deserialize<'i> for f64 {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<f64> {
            fn negative(&mut self, n: i64, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f64);
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _c: &mut dyn Context) -> Result<()> {
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

impl<'i> Deserialize<'i> for f32 {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i> Visitor<'i> for Place<f32> {
            fn negative(&mut self, n: i64, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f32);
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(n as f32);
                Ok(())
            }

            fn double(&mut self, n: f64) -> Result<()> {
                if n <= (f32::MAX as f64) && n >= (f32::MIN as f64) {
                    self.out = Some(n as f32);
                    Ok(())
                } else {
                    Err(Error)
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

impl<'i, T: Deserialize<'i>> Deserialize<'i> for Box<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, T: Deserialize<'i>> Visitor<'i> for Place<Box<T>> {
            fn null(&mut self, context: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).null(context)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).boolean(b)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn string(&mut self, s: &'i str, context: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).string(s, context)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn negative(&mut self, n: i64, context: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).negative(n, context)?;
                self.out = Some(Box::new(out.unwrap()));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, context: &mut dyn Context) -> Result<()> {
                let mut out = None;
                Deserialize::begin(&mut out).nonnegative(n, context)?;
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

            fn seq(&mut self, context: &mut dyn Context) -> Result<Box<dyn Seq<'i> + '_>> {
                let mut value = Box::new(None);
                let ptr = careful!(&mut *value as &mut Option<T>);
                Ok(Box::new(BoxSeq {
                    out: &mut self.out,
                    value,
                    seq: Deserialize::begin(ptr).seq(context)?,
                }))
            }

            fn map(&mut self, context: &mut dyn Context) -> Result<Box<dyn Map<'i> + '_>> {
                let mut value = Box::new(None);
                let ptr = careful!(&mut *value as &mut Option<T>);
                Ok(Box::new(BoxMap {
                    out: &mut self.out,
                    value,
                    map: Deserialize::begin(ptr).map(context)?,
                }))
            }
        }

        struct BoxSeq<'a, 'i: 'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: Box<Option<T>>,
            seq: Box<dyn Seq<'i> + 'a>,
        }

        impl<'a, 'i: 'a, T: Deserialize<'i>> Seq<'i> for BoxSeq<'i, 'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor<'i>> {
                self.seq.element()
            }

            fn finish(&mut self) -> Result<()> {
                self.seq.finish()?;
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        struct BoxMap<'a, 'i: 'a, T: 'a> {
            out: &'a mut Option<Box<T>>,
            value: Box<Option<T>>,
            map: Box<dyn Map<'i> + 'a>,
        }

        impl<'a, 'i: 'a, T: Deserialize<'i>> Map<'i> for BoxMap<'i, 'a, T> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'i>> {
                self.map.key(k)
            }

            fn finish(&mut self) -> Result<()> {
                self.map.finish()?;
                *self.out = Some(Box::new(self.value.take().unwrap()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'i, T: Deserialize<'i>> Deserialize<'i> for Option<T> {
    #[inline]
    fn default() -> Option<Self> {
        Some(None)
    }
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, T: Deserialize<'i>> Visitor<'i> for Place<Option<T>> {
            fn null(&mut self, _c: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Ok(())
            }

            fn boolean(&mut self, b: bool) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).boolean(b)
            }

            fn string(&mut self, s: &'i str, context: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).string(s, context)
            }

            fn negative(&mut self, n: i64, context: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).negative(n, context)
            }

            fn nonnegative(&mut self, n: u64, context: &mut dyn Context) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).nonnegative(n, context)
            }

            fn single(&mut self, n: f32) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).single(n)
            }

            fn double(&mut self, n: f64) -> Result<()> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).double(n)
            }

            fn seq(&mut self, context: &mut dyn Context) -> Result<Box<dyn Seq<'i> + '_>> {
                self.out = Some(None);
                let o = self.out.as_mut().unwrap(); // &mut Option<T>
                let v = careful!(Deserialize::begin(o) as &mut dyn Visitor<'i>);
                v.seq(context) // Result<Box<dyn Seq<'i> + '_>>
            }

            fn map(&mut self, context: &mut dyn Context) -> Result<Box<dyn Map<'i> + '_>> {
                self.out = Some(None);
                Deserialize::begin(self.out.as_mut().unwrap()).map(context)
            }
        }

        Place::new(out)
    }
}

impl<'i, A: Deserialize<'i>, B: Deserialize<'i>> Deserialize<'i> for (A, B) {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, A: Deserialize<'i>, B: Deserialize<'i>> Visitor<'i> for Place<(A, B)> {
            fn seq(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Seq<'i> + '_>> {
                Ok(Box::new(TupleBuilder {
                    out: &mut self.out,
                    tuple: (None, None),
                }))
            }
        }

        struct TupleBuilder<'a, A: 'a, B: 'a> {
            out: &'a mut Option<(A, B)>,
            tuple: (Option<A>, Option<B>),
        }

        impl<'i, 'a, A: Deserialize<'i>, B: Deserialize<'i>> Seq<'i> for TupleBuilder<'a, A, B> {
            fn element(&mut self) -> Result<&mut dyn Visitor<'i>> {
                if self.tuple.0.is_none() {
                    Ok(Deserialize::begin(&mut self.tuple.0))
                } else if self.tuple.1.is_none() {
                    Ok(Deserialize::begin(&mut self.tuple.1))
                } else {
                    Err(Error)
                }
            }

            fn finish(&mut self) -> Result<()> {
                if let (Some(a), Some(b)) = (self.tuple.0.take(), self.tuple.1.take()) {
                    *self.out = Some((a, b));
                    Ok(())
                } else {
                    Err(Error)
                }
            }
        }

        Place::new(out)
    }
}

impl<'i, T: Deserialize<'i>> Deserialize<'i> for Vec<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, T: Deserialize<'i>> Visitor<'i> for Place<Vec<T>> {
            fn seq(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Seq<'i> + '_>> {
                Ok(Box::new(VecBuilder {
                    out: &mut self.out,
                    vec: Vec::new(),
                    element: None,
                }))
            }
        }

        struct VecBuilder<'a, T: 'a> {
            out: &'a mut Option<Vec<T>>,
            vec: Vec<T>,
            element: Option<T>,
        }

        impl<'a, T> VecBuilder<'a, T> {
            fn shift(&mut self) {
                if let Some(e) = self.element.take() {
                    self.vec.push(e);
                }
            }
        }

        impl<'i, 'a, T: Deserialize<'i>> Seq<'i> for VecBuilder<'a, T> {
            fn element(&mut self) -> Result<&mut dyn Visitor<'i>> {
                self.shift();
                Ok(Deserialize::begin(&mut self.element))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.vec, Vec::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'i, K, V, H> Deserialize<'i> for HashMap<K, V, H>
where
    K: FromStr + Hash + Eq,
    V: Deserialize<'i>,
    H: BuildHasher + Default,
{
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, K, V, H> Visitor<'i> for Place<HashMap<K, V, H>>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize<'i>,
            H: BuildHasher + Default,
        {
            fn map(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Map<'i> + '_>> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: HashMap::with_hasher(H::default()),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a, H: 'a> {
            out: &'a mut Option<HashMap<K, V, H>>,
            map: HashMap<K, V, H>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K: Hash + Eq, V, H: BuildHasher> MapBuilder<'a, K, V, H> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'i, 'a, K, V, H> Map<'i> for MapBuilder<'a, K, V, H>
        where
            K: FromStr + Hash + Eq,
            V: Deserialize<'i>,
            H: BuildHasher + Default,
        {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'i>> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(Error),
                });
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                let substitute = HashMap::with_hasher(H::default());
                *self.out = Some(mem::replace(&mut self.map, substitute));
                Ok(())
            }
        }

        Place::new(out)
    }
}

impl<'i, K: FromStr + Ord, V: Deserialize<'i>> Deserialize<'i> for BTreeMap<K, V> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'i> {
        impl<'i, K: FromStr + Ord, V: Deserialize<'i>> Visitor<'i> for Place<BTreeMap<K, V>> {
            fn map(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Map<'i> + '_>> {
                Ok(Box::new(MapBuilder {
                    out: &mut self.out,
                    map: BTreeMap::new(),
                    key: None,
                    value: None,
                }))
            }
        }

        struct MapBuilder<'a, K: 'a, V: 'a> {
            out: &'a mut Option<BTreeMap<K, V>>,
            map: BTreeMap<K, V>,
            key: Option<K>,
            value: Option<V>,
        }

        impl<'a, K: Ord, V> MapBuilder<'a, K, V> {
            fn shift(&mut self) {
                if let (Some(k), Some(v)) = (self.key.take(), self.value.take()) {
                    self.map.insert(k, v);
                }
            }
        }

        impl<'i, 'a, K: FromStr + Ord, V: Deserialize<'i>> Map<'i> for MapBuilder<'a, K, V> {
            fn key(&mut self, k: &str) -> Result<&mut dyn Visitor<'i>> {
                self.shift();
                self.key = Some(match K::from_str(k) {
                    Ok(key) => key,
                    Err(_) => return Err(Error),
                });
                Ok(Deserialize::begin(&mut self.value))
            }

            fn finish(&mut self) -> Result<()> {
                self.shift();
                *self.out = Some(mem::replace(&mut self.map, BTreeMap::new()));
                Ok(())
            }
        }

        Place::new(out)
    }
}
