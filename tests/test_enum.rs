use knocknoc::json;
use knocknoc::export::Cow;
use knocknoc::de::{Deserialize, Visitor};
use knocknoc::ser::{Serialize, Fragment};

#[derive(Debug, PartialEq)]
enum E {
    W { a: i32, b: i32 },
    X(i32, i32),
    Y(i32),
    Z,
}

impl Serialize for E {
    fn begin(&self, _c: &dyn knocknoc::ser::Context) -> Fragment {
        match self {
            E::W { a, b } => {
                #[derive(knocknoc::Serialize)]
                struct Proxy<'p> { a: &'p i32, b: &'p i32 }

                struct __Map<'__a> {
                    data: Proxy<'__a>,
                    state: knocknoc::export::usize,
                }
    
                impl<'__a> knocknoc::ser::Map for __Map<'__a> {
                    fn next(&mut self) -> knocknoc::export::Option<(knocknoc::export::Cow<knocknoc::export::str>, &dyn knocknoc::Serialize)> {
                        let __state = self.state;
                        self.state = __state + 1;
                        match __state {
                            0 => knocknoc::export::Some((
                                knocknoc::export::Cow::Borrowed("W"),
                                &self.data,
                            )),
                            _ => knocknoc::export::None,
                        }
                    }
                }

                knocknoc::ser::Fragment::Map(knocknoc::export::Box::new(__Map {
                    data: Proxy { a, b },
                    state: 0,
                }))
            },
            E::X(v0, v1) => {
                struct __Map<'__a> {
                    data: (&'__a i32, &'__a i32),
                    state: knocknoc::export::usize,
                }
    
                impl<'__a> knocknoc::ser::Map for __Map<'__a> {
                    fn next(&mut self) -> knocknoc::export::Option<(knocknoc::export::Cow<knocknoc::export::str>, &dyn knocknoc::Serialize)> {
                        let __state = self.state;
                        self.state = __state + 1;
                        match __state {
                            0 => knocknoc::export::Some((
                                knocknoc::export::Cow::Borrowed("X"),
                                &self.data,
                            )),
                            _ => knocknoc::export::None,
                        }
                    }
                }

                knocknoc::ser::Fragment::Map(knocknoc::export::Box::new(__Map {
                    data: (v0, v1),
                    state: 0,
                }))
            },
            E::Y(v0) => {
                struct __Map<'__a> {
                    data: &'__a i32,
                    state: knocknoc::export::usize,
                }
    
                impl<'__a> knocknoc::ser::Map for __Map<'__a> {
                    fn next(&mut self) -> knocknoc::export::Option<(knocknoc::export::Cow<knocknoc::export::str>, &dyn knocknoc::Serialize)> {
                        let __state = self.state;
                        self.state = __state + 1;
                        match __state {
                            0 => knocknoc::export::Some((
                                knocknoc::export::Cow::Borrowed("Y"),
                                &self.data,
                            )),
                            _ => knocknoc::export::None,
                        }
                    }
                }

                knocknoc::ser::Fragment::Map(knocknoc::export::Box::new(__Map {
                    data: v0,
                    state: 0,
                }))
            },
            E::Z => Fragment::Str(Cow::Borrowed("Z")),
        }
    }
}

knocknoc::make_place!(Place);

impl<'__i> Deserialize<'__i> for E {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor<'__i> {
        impl<'__i> Visitor<'__i> for Place<E> {
            fn string(&mut self, s: &str, _c: &mut dyn knocknoc::de::Context) -> knocknoc::Result<()> {
                match s {
                    "Z" => { self.out = Some(E::Z); Ok(()) },
                    _ => Err(knocknoc::Error),
                }
            }
        
            fn map(&mut self, _c: &mut dyn knocknoc::de::Context) -> knocknoc::Result<Box<dyn knocknoc::de::Map<'__i> + '_>> {
                #[derive(knocknoc::Deserialize)]
                struct W {
                    a: i32, b: i32,
                }
                
                #[derive(PartialEq)]
                enum __Var {
                    None, W, X, Y
                }

                #[allow(non_snake_case)]
                struct __State<'__a> {
                    state: __Var,
                    W: knocknoc::export::Option<W>,
                    X: knocknoc::export::Option<(i32, i32)>,
                    Y: knocknoc::export::Option<i32>,
                    __out: &'__a mut knocknoc::export::Option<E>,
                }
    
                impl<'__a, '__i> knocknoc::de::Map<'__i> for __State<'__a> {
                    fn key(&mut self, __k: &knocknoc::export::str) -> knocknoc::Result<&mut dyn knocknoc::de::Visitor<'__i>> {
                        if self.state != __Var::None {
                            knocknoc::export::Ok(knocknoc::de::Visitor::ignore())
                        } else {
                            match __k {
                                "W" => {
                                    self.state = __Var::W;
                                    knocknoc::export::Ok(knocknoc::Deserialize::begin(&mut self.W))
                                },
                                "X" => {
                                    self.state = __Var::X;
                                    knocknoc::export::Ok(knocknoc::Deserialize::begin(&mut self.X))
                                },
                                "Y" => {
                                    self.state = __Var::Y;
                                    knocknoc::export::Ok(knocknoc::Deserialize::begin(&mut self.Y))
                                },
                                _ => knocknoc::export::Ok(knocknoc::de::Visitor::ignore()),
                            }
                        }
                    }
    
                    fn finish(&mut self) -> knocknoc::Result<()> {
                        match self.state {
                            __Var::W => *self.__out = Some(self.W.as_ref().map(|w| E::W { a: w.a, b: w.b }).ok_or(knocknoc::Error)?),
                            __Var::X => *self.__out = Some(self.X.map(|x| E::X(x.0, x.1)).ok_or(knocknoc::Error)?),
                            __Var::Y => *self.__out = Some(self.Y.map(E::Y).ok_or(knocknoc::Error)?),
                            _ => { return Err(knocknoc::Error); },
                        }
                        Ok(())
                    }
                }

                Ok(Box::new(__State {
                    state: __Var::None,
                    W: None,
                    X: None,
                    Y: None,
                    __out: &mut self.out,
                }))
            }
        }
        Place::new(out)
    }
}

#[test]
fn test_enum() {
    let cases = &[
        (E::W { a: 0, b: 0 }, r#"{"W":{"a":0,"b":0}}"#),
        (E::X(0, 0), r#"{"X":[0,0]}"#),
        (E::Y(0), r#"{"Y":0}"#),
        (E::Z, r#""Z""#),
    ];
    
    for (val, expected) in cases {
        let actual = json::to_string(val, &mut ());
        assert_eq!(actual, *expected);
    }

    for (expected, val) in cases {
        let mut val = val.to_string();
        let actual: E = json::from_str(&mut val, &mut ()).unwrap();
        assert_eq!(actual, *expected);
    }
}
