use crate::de::{Map, Seq, Visitor, Context};
use crate::error::Result;

impl<'i> dyn Visitor<'i> {
    pub fn ignore<'a>() -> &'a mut dyn Visitor<'i> {
        careful!(&mut Ignore as &mut Ignore)
    }
}

struct Ignore;

impl<'i> Visitor<'i> for Ignore {
    fn null(&mut self, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    fn boolean(&mut self, _b: bool) -> Result<()> {
        Ok(())
    }

    fn string(&mut self, _s: &'i str, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }


    fn bytes(&mut self, _b: &'i [u8], _c: &mut dyn Context) -> Result<()> {
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

    fn seq(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Seq<'i> + '_>> {
        Ok(Box::new(Ignore))
    }

    fn map(&mut self, _c: &mut dyn Context) -> Result<Box<dyn Map<'i> + '_>> {
        Ok(Box::new(Ignore))
    }
}

impl<'i> Seq<'i> for Ignore {
    fn element(&mut self) -> Result<&mut dyn Visitor<'i>> {
        Ok(careful!(&mut Ignore as &mut Ignore))
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<'i> Map<'i> for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut dyn Visitor<'i>> {
        Ok(careful!(&mut Ignore as &mut Ignore))
    }

    fn finish(&mut self) -> Result<()> {
        Ok(())
    }
}
