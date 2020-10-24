use crate::error::Result;
use crate::export::{HandleId, Hint};
use crate::Place;
use crate::{de, ser};
use crate::{Deserialize, Serialize};
use bevy::prelude::*;

impl de::Context for &AssetServer {
    // fn entity(&mut self, e: Hint) -> crate::Result<Entity> {
    //     let _ = e;
    //     Err(crate::Error::not_expected("entity"))?
    // }

    fn asset(&mut self, a: Hint) -> crate::Result<HandleId> {
        match a {
            Hint::Null => todo!("null assets aren't supported"),
            Hint::Str(p) => Ok((*self).load_untyped(p)?),
            Hint::Bytes(b) => Ok(HandleId(uuid::Uuid::from_slice(b).expect("invalid uuid"))),
        }
    }
}

impl Serialize for HandleId {
    fn begin(&self, _: ser::Visitor, _: &mut dyn ser::Context) -> ser::Done {
        todo!("handle serialization not supported")
    }
}

impl<'de> Deserialize<'de> for HandleId {
    fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor<'de> {
        impl<'de> de::Visitor<'de> for Place<HandleId> {
            fn null(&mut self, c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(c.asset(Hint::Null)?);
                Ok(())
            }

            fn string(&mut self, s: &'de str, c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(c.asset(Hint::Str(s))?);
                Ok(())
            }

            fn bytes(&mut self, b: &'de [u8], c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(c.asset(Hint::Bytes(b))?);
                Ok(())
            }
        }
        Place::new(out)
    }
}

impl<T: 'static> Serialize for Handle<T> {
    fn begin(&self, _: ser::Visitor, _: &mut dyn ser::Context) -> ser::Done {
        todo!("handle serialization not supported")
    }
}

// TODO: may yield invalid asset if the type doesn't match
impl<'de, T: 'static> Deserialize<'de> for Handle<T> {
    fn begin(out: &mut Option<Self>) -> &mut dyn de::Visitor<'de> {
        impl<'de, T: 'static> de::Visitor<'de> for Place<Handle<T>> {
            fn null(&mut self, c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Handle::from_id(c.asset(Hint::Null)?));
                Ok(())
            }

            fn string(&mut self, s: &'de str, c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Handle::from_id(c.asset(Hint::Str(s))?));
                Ok(())
            }

            fn bytes(&mut self, b: &'de [u8], c: &mut dyn de::Context) -> Result<()> {
                self.out = Some(Handle::from_id(c.asset(Hint::Bytes(b))?));
                Ok(())
            }
        }
        Place::new(out)
    }
}
