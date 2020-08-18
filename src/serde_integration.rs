//! Serde integration (Optional)

use crate::bytes::guess_align_of;
use crate::export::Cow;
use crate::ser::Fragment;
use crate::{Error, Result};
use serde1 as serde;

struct Warp<T>(T);

impl serde::ser::Error for Error {}

impl<T: serde::Serialize> crate::Serialize for Warp<T> {
    fn begin(&self, context: &dyn crate::ser::Context) -> Fragment {
        struct FragSerializer<'a>(Fragment<'a>);

        impl<'b, 'a: 'b> serde::Serializer for &'b mut FragSerializer<'a> {
            type Ok = ();
            type Error = Error;
            type SerializeSeq = Self;
            type SerializeTuple = Self;
            type SerializeTupleStruct = Self;
            type SerializeTupleVariant = Self;
            type SerializeMap = Self;
            type SerializeStruct = Self;
            type SerializeStructVariant = Self;

            fn serialize_bool(self, v: bool) -> Result<()> {
                self.0 = Fragment::Bool(v);
                Ok(())
            }

            fn serialize_i8(self, v: i8) -> Result<()> {
                self.0 = Fragment::I8(v);
                Ok(())
            }

            fn serialize_i16(self, v: i16) -> Result<()> {
                self.0 = Fragment::I32(v as i32);
                Ok(())
            }

            fn serialize_i32(self, v: i32) -> Result<()> {
                self.0 = Fragment::I32(v);
                Ok(())
            }

            fn serialize_i64(self, v: i64) -> Result<()> {
                self.0 = Fragment::I64(v);
                Ok(())
            }

            fn serialize_u8(self, v: i8) -> Result<()> {
                self.0 = Fragment::U8(v);
                Ok(())
            }

            fn serialize_u16(self, v: i16) -> Result<()> {
                self.0 = Fragment::U32(v as i32);
                Ok(())
            }

            fn serialize_u32(self, v: i32) -> Result<()> {
                self.0 = Fragment::U32(v);
                Ok(())
            }

            fn serialize_u64(self, v: i64) -> Result<()> {
                self.0 = Fragment::U64(v);
                Ok(())
            }

            fn serialize_f32(self, v: f32) -> Result<()> {
                self.0 = Fragment::F32(v);
                Ok(())
            }

            fn serialize_f64(self, v: f64) -> Result<()> {
                self.0 = Fragment::F64(v);
                Ok(())
            }

            fn serialize_char(self, v: char) -> Result<()> {
                todo!()
            }

            fn serialize_str(self, v: &str) -> Result<()> {
                self.0 = Fragment::Str(Cow::Borrowed(v));
                Ok(())
            }

            fn serialize_bytes(self, v: &[u8]) -> Result<()> {
                self.0 = Fragment::Bin {
                    bytes: Cow::Borrowed(v),
                    align: guess_align_of(v.as_ptr()),
                };
                Ok(())
            }

            fn serialize_none(self) -> Result<()> {
                Ok(())
            }

            fn serialize_some<T>(self, value: &T) -> Result<()>
            where
                T: ?Sized + serde::Serialize,
            {
                todo!()
            }

            fn serialize_unit(self) -> Result<()> {
                todo!()
            }

            fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
                todo!()
            }

            fn serialize_unit_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                variant: &'static str,
            ) -> Result<()> {
                todo!()
            }

            fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
            where
                T: ?Sized + serde::Serialize,
            {
                todo!()
            }

            fn serialize_newtype_variant<T>(
                self,
                _name: &'static str,
                _variant_index: u32,
                variant: &'static str,
                value: &T,
            ) -> Result<()>
            where
                T: ?Sized + serde::Serialize,
            {
                todo!()
            }

            fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
                todo!()
            }

            fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
                todo!()
            }

            fn serialize_tuple_struct(
                self,
                _name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeTupleStruct> {
                todo!()
            }

            fn serialize_tuple_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeTupleVariant> {
                todo!()
            }

            // Maps are represented in JSON as `{ K: V, K: V, ... }`.
            fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
                todo!()
            }

            fn serialize_struct(
                self,
                _name: &'static str,
                len: usize,
            ) -> Result<Self::SerializeStruct> {
                todo!()
            }

            fn serialize_struct_variant(
                self,
                _name: &'static str,
                _variant_index: u32,
                variant: &'static str,
                _len: usize,
            ) -> Result<Self::SerializeStructVariant> {
                todo!()
            }
        }

        let mut f = FragSerializer(Fragment::Null);
        self.0.serialize(&mut f).is_ok();
        f.0
    }
}
