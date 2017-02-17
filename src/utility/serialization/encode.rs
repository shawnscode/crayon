use std::marker::PhantomData;
use byteorder::WriteBytesExt;
use std::io::Write;
use std::u32;

use serde;
use byteorder;

use utility::memory::variant::VariantChar;
use super::error::{Result, Error, ErrorKind};

/// An Serializer that encodes values directly into a Writer.
///
/// This struct should not be used often.
/// For most cases, prefer the `encode_into` function.
pub struct Serializer<W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    writer: W,
    _phantom: PhantomData<E>,
}

impl<W, E> Serializer<W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    pub fn new(w: W) -> Serializer<W, E> {
        Serializer {
            writer: w,
            _phantom: PhantomData,
        }
    }

    fn write_variant_uint(&mut self, v: usize) -> Result<()> {
        assert!(v < u32::MAX as usize, "variant uint doesn't fit in a u32");

        if v < 0xFF {
            self.writer.write_u8(v as u8).map_err(Into::into)
        } else {
            let result: Result<()> = self.writer.write_u8(0xFF).map_err(Into::into);
            result?;
            self.writer.write_u32::<E>(v as u32).map_err(Into::into)
        }
    }
}

impl<'a, W, E> serde::Serializer for &'a mut Serializer<W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Compound<'a, W, E>;
    type SerializeTuple = Compound<'a, W, E>;
    type SerializeTupleStruct = Compound<'a, W, E>;
    type SerializeTupleVariant = Compound<'a, W, E>;
    type SerializeMap = Compound<'a, W, E>;
    type SerializeStruct = Compound<'a, W, E>;
    type SerializeStructVariant = Compound<'a, W, E>;

    fn serialize_unit(self) -> Result<()> {
        Ok(())
    }

    fn serialize_unit_struct(self, _: &'static str) -> Result<()> {
        Ok(())
    }

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.writer.write_u8(if v { 1 } else { 0 }).map_err(Into::into)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.writer.write_u8(v).map_err(Into::into)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.writer.write_u16::<E>(v).map_err(Into::into)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.writer.write_u32::<E>(v).map_err(Into::into)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.writer.write_u64::<E>(v).map_err(Into::into)
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.writer.write_i8(v).map_err(Into::into)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.writer.write_i16::<E>(v).map_err(Into::into)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.writer.write_i32::<E>(v).map_err(Into::into)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.writer.write_i64::<E>(v).map_err(Into::into)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.writer.write_f32::<E>(v).map_err(Into::into)
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.writer.write_f64::<E>(v).map_err(Into::into)
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_variant_uint(v.len())?;
        self.writer.write_all(v.as_bytes()).map_err(Into::into)
    }

    fn serialize_char(self, c: char) -> Result<()> {
        self.writer.write_all(VariantChar::from(c).as_slice()).map_err(Into::into)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_variant_uint(v.len())?;
        self.writer.write_all(v).map_err(Into::into)
    }

    fn serialize_none(self) -> Result<()> {
        self.writer.write_u8(0).map_err(Into::into)
    }

    fn serialize_some<T: ?Sized>(self, v: &T) -> Result<()>
        where T: serde::Serialize
    {
        self.writer.write_u8(1)?;
        v.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        let len = len.ok_or(ErrorKind::SequenceMustHaveLength)?;
        self.write_variant_uint(len)?;
        Ok(Compound { ser: self })
    }

    fn serialize_seq_fixed_size(self, _len: usize) -> Result<Self::SerializeSeq> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_struct(self,
                              _name: &'static str,
                              _len: usize)
                              -> Result<Self::SerializeTupleStruct> {
        Ok(Compound { ser: self })
    }

    fn serialize_tuple_variant(self,
                               _name: &'static str,
                               variant_index: usize,
                               _variant: &'static str,
                               _len: usize)
                               -> Result<Self::SerializeTupleVariant> {
        self.write_variant_uint(variant_index)?;
        Ok(Compound { ser: self })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        let len = len.ok_or(ErrorKind::SequenceMustHaveLength)?;
        self.serialize_u64(len as u64)?;
        Ok(Compound { ser: self })
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Ok(Compound { ser: self })
    }

    fn serialize_struct_variant(self,
                                _name: &'static str,
                                variant_index: usize,
                                _variant: &'static str,
                                _len: usize)
                                -> Result<Self::SerializeStructVariant> {
        self.write_variant_uint(variant_index)?;
        Ok(Compound { ser: self })
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized>(self,
                                            _name: &'static str,
                                            variant_index: usize,
                                            _variant: &'static str,
                                            value: &T)
                                            -> Result<()>
        where T: serde::ser::Serialize
    {
        self.write_variant_uint(variant_index)?;
        value.serialize(self)
    }

    fn serialize_unit_variant(self,
                              _name: &'static str,
                              variant_index: usize,
                              _variant: &'static str)
                              -> Result<()> {
        self.write_variant_uint(variant_index)
    }
}

#[doc(hidden)]
pub struct Compound<'a, W, E>
    where W: 'a + Write,
          E: 'a + byteorder::ByteOrder
{
    ser: &'a mut Serializer<W, E>,
}

impl<'a, W, E> serde::ser::SerializeSeq for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeTuple for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeTupleStruct for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeTupleVariant for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeMap for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<K: ?Sized>(&mut self, value: &K) -> Result<()>
        where K: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn serialize_value<V: ?Sized>(&mut self, value: &V) -> Result<()>
        where V: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeStruct for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}

impl<'a, W, E> serde::ser::SerializeStructVariant for Compound<'a, W, E>
    where W: Write,
          E: byteorder::ByteOrder
{
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T: ?Sized>(&mut self, _key: &'static str, value: &T) -> Result<()>
        where T: serde::ser::Serialize
    {
        value.serialize(&mut *self.ser)
    }

    #[inline]
    fn end(self) -> Result<()> {
        Ok(())
    }
}