use knocknoc::json;
use knocknoc::Result;
use knocknoc::export::Cow;
use knocknoc::de::{self, Deserialize, Visitor};
use knocknoc::ser::{self, Serialize, Fragment};

#[derive(Debug, PartialEq)]
struct Bytes(Vec<u8>);

impl Serialize for Bytes {
    fn begin(&self, _c: &dyn ser::Context) -> Fragment {
        Fragment::Bin(Cow::Borrowed(self.0.as_slice()))
    }
}

knocknoc::make_place!(Place);

impl Deserialize for Bytes {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl Visitor for Place<Bytes> {
            fn bytes(&mut self, b: &[u8], _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bytes(b.to_vec()));
                Ok(())
            }
        }
        Place::new(out)
    }
}

#[test]
fn test_binhint() {
    let cases = &[
        (Bytes(vec![2, 0, 3, 4]), r#""02000304\u0010""#),
    ];
    
    for (val, expected) in cases {
        let actual = json::to_string(val, &mut ());
        assert_eq!(actual, *expected);
    }

    for (expected, val) in cases {
        let actual: Bytes = json::from_str(val, &mut ()).unwrap();
        assert_eq!(actual, *expected);
    }
}
