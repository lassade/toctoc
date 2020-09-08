use toctoc::de::{self, Deserialize};
use toctoc::json;
use toctoc::ser::{self, Done, Serialize};

#[derive(Debug, PartialEq)]
enum E {
    W { a: i32, b: i32 },
    X(i32, i32),
    Y(i32),
    Z,
}

impl Serialize for E {
    fn begin(&self, v: ser::Visitor, c: &dyn ser::Context) -> Done {
        match self {
            E::W { a, b } => {
                #[derive(toctoc::Serialize)]
                struct Inner<'a> {
                    a: &'a i32,
                    b: &'a i32,
                };

                v.map().field("W", &Inner { a: a, b: b }, c).done()
            }
            E::X(v0, v1) => v.map().field("X", &(v0, v1), c).done(),
            E::Y(v0) => v.map().field("Y", &(v0), c).done(),
            E::Z => v.string("Z"),
        }
    }
}

toctoc::make_place!(Place);

#[allow(unused_parens)]
impl<'de> Deserialize<'de> for E {
    fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor<'de> {
        impl<'de> de::Visitor<'de> for Place<E> {
            fn string(&mut self, s: &str, _: &mut dyn de::Context) -> toctoc::Result<()> {
                match s {
                    "Z" => {
                        self.out = Some(E::Z);
                        Ok(())
                    }
                    __variant => Err(toctoc::Error::unknown_variant(__variant)),
                }
            }

            fn map(
                &mut self,
                m: &mut dyn de::Map<'de>,
                c: &mut dyn de::Context,
            ) -> toctoc::Result<()> {
                match m.next()? {
                    Some("W") => {
                        #[derive(toctoc::Deserialize)]
                        struct Inner {
                            a: i32,
                            b: i32,
                        }

                        let mut v = None;
                        m.visit(Inner::begin(&mut v), c)?;
                        let v = v.unwrap();
                        self.out = Some(E::W { a: v.a, b: v.b });
                    }
                    Some("X") => {
                        let mut v: Option<(i32, i32)> = None;
                        m.visit(toctoc::de::Deserialize::begin(&mut v), c)?;
                        let v = v.unwrap();
                        self.out = Some(E::X(v.0, v.1));
                    }
                    Some("Y") => {
                        let mut v: Option<(i32)> = None;
                        m.visit(toctoc::de::Deserialize::begin(&mut v), c)?;
                        let v = v.unwrap();
                        self.out = Some(E::Y(v));
                    }
                    _ => m.visit(toctoc::de::Visitor::ignore(), c)?,
                }

                Ok(())
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
