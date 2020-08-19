use crate::de::{Context, Map, Seq, Visitor};
use crate::error::Result;

impl<'de> dyn Visitor<'de> {
    pub fn ignore<'a>() -> &'a mut dyn Visitor<'de> {
        careful!(&mut Ignore as &mut Ignore)
    }
}

struct Ignore;

impl<'de> Visitor<'de> for Ignore {
    fn null(&mut self, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn boolean(&mut self, _b: bool) -> Result<()> {
        Ok(())
    }

    fn string(&mut self, _s: &'de str, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn bytes(&mut self, _b: &'de [u8], _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn negative(&mut self, _n: i64, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn nonnegative(&mut self, _n: u64, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn single(&mut self, _n: f32) -> Result<()> {
        Ok(())
    }

    fn double(&mut self, _n: f64) -> Result<()> {
        Ok(())
    }

    fn seq(&mut self, s: &mut dyn Seq<'de>, c: &mut dyn Context) -> Result<()> {
        while s.visit(&mut Ignore, c)? {}
        Ok(())
    }

    fn map(&mut self, m: &mut dyn Map<'de>, c: &mut dyn Context) -> Result<()> {
        while let Some(_) = m.next()? {
            m.visit(&mut Ignore, c)?
        }
        Ok(())
    }
}
