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

    fn seq<'a>(&'a mut self) -> Result<Box<dyn Seq<'de> + 'a>>
    where
        'de: 'a,
    {
        Ok(Box::new(Ignore))
    }

    fn map<'a>(&'a mut self) -> Result<Box<dyn Map<'de> + 'a>>
    where
        'de: 'a,
    {
        Ok(Box::new(Ignore))
    }
}

impl<'de> Seq<'de> for Ignore {
    fn element(&mut self) -> Result<&mut dyn Visitor<'de>> {
        Ok(careful!(&mut Ignore as &mut Ignore))
    }

    fn finish(&mut self, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }
}

impl<'de> Map<'de> for Ignore {
    fn key(&mut self, _k: &str) -> Result<&mut dyn Visitor<'de>> {
        Ok(careful!(&mut Ignore as &mut Ignore))
    }

    fn finish(&mut self, _c: &mut dyn Context) -> Result<()> {
        Ok(())
    }
}
