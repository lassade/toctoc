use std::borrow::Cow;

use crate::de::{self, Deserialize, Visitor};
use crate::error::{Result, Error};
use crate::ser::{self, Fragment, Serialize};
use crate::Place;

/// Decorate binary data to be properly (de)serialized both
/// by json or bson formats
pub struct Bin<'a>(pub Cow<'a, [u8]>);

impl<'a> Serialize for Bin<'a> {
    fn begin(&self, _c: &dyn ser::Context) -> Fragment {
        Fragment::Bin(Cow::Borrowed(self.0.as_ref()))
    }
}

impl<'a> Deserialize for Bin<'a> {
    fn begin(out: &mut Option<Self>) -> &mut dyn Visitor {
        impl<'a> Visitor for Place<Bin<'a>> {
            fn null(&mut self, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bin(Cow::Owned(vec![])));
                Ok(())
            }
            
            fn string(&mut self, s: &str, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bin(Cow::Owned(hex::decode(s).map_err(|_| Error)?)));
                Ok(())
            }

            fn negative(&mut self, n: i64, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bin(Cow::Owned(n.to_le_bytes().to_vec())));
                Ok(())
            }

            fn nonnegative(&mut self, n: u64, _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bin(Cow::Owned(n.to_le_bytes().to_vec())));
                Ok(())
            }
        
            fn bytes(&mut self, b: &[u8], _c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Bin(Cow::Owned(b.to_owned())));
                Ok(())
            }
        }

        Place::new(out)
    }
}