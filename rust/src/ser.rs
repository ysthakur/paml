use serde::{ser, Serialize};

use crate::error::{Error, Result};

pub struct Serializer {
  output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
  T: Serialize,
{
  let mut serializer = Serializer {
    output: String::new(),
  };
  value.serialize(&mut serializer)?;
  Ok(serializer.output)
}

/// Write the type for the value that follows
#[must_use]
fn serialize_type(s: &mut Serializer, typ: &str) -> Result<()> {
  s.output += &format!("~{} ", typ);
  Ok(())
}

impl<'a> ser::Serializer for &'a mut Serializer {
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
    self.output += if v { "true" } else { "false" };
    Ok(())
  }

  fn serialize_i8(self, v: i8) -> Result<()> {
    self.serialize_i64(i64::from(v))
  }

  fn serialize_i16(self, v: i16) -> Result<()> {
    self.serialize_i64(i64::from(v))
  }

  fn serialize_i32(self, v: i32) -> Result<()> {
    self.serialize_i64(i64::from(v))
  }

  fn serialize_i64(self, v: i64) -> Result<()> {
    self.output += &v.to_string();
    Ok(())
  }

  fn serialize_u8(self, v: u8) -> Result<()> {
    self.serialize_u64(u64::from(v))
  }

  fn serialize_u16(self, v: u16) -> Result<()> {
    self.serialize_u64(u64::from(v))
  }

  fn serialize_u32(self, v: u32) -> Result<()> {
    self.serialize_u64(u64::from(v))
  }

  fn serialize_u64(self, v: u64) -> Result<()> {
    self.output += &v.to_string();
    Ok(())
  }

  fn serialize_f32(self, v: f32) -> Result<()> {
    self.serialize_f64(f64::from(v))
  }

  fn serialize_f64(self, v: f64) -> Result<()> {
    self.output += &v.to_string();
    Ok(())
  }

  fn serialize_char(self, v: char) -> Result<()> {
    self.serialize_str(&v.to_string())
  }

  fn serialize_str(self, v: &str) -> Result<()> {
    self.output += "\"";
    self.output += &v
      .replace("\\", "\\\\")
      .replace("\"", "\\\"")
      .replace("\n", "\\n")
      .replace("\r", "\\r");
    self.output += "\"";
    Ok(())
  }

  fn serialize_bytes(self, v: &[u8]) -> Result<()> {
    use ser::SerializeSeq;
    let mut s = self.serialize_seq(Some(v.len()))?;
    for b in v {
      s.serialize_element(b)?;
    }
    s.end()
  }

  fn serialize_none(self) -> Result<()> {
    self.serialize_unit_variant("Option", 0, "None")
  }

  fn serialize_some<T: ?Sized>(self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    self.serialize_newtype_variant("Option", 0, "Some", value)
  }

  fn serialize_unit(self) -> Result<()> {
    self.output += "null";
    Ok(())
  }

  fn serialize_unit_struct(self, name: &'static str) -> Result<()> {
    use ser::SerializeStruct;
    let s = self.serialize_struct(name, 0)?;
    s.end()
  }

  fn serialize_unit_variant(
    self,
    name: &'static str,
    _variant_index: u32,
    _variant: &'static str,
  ) -> Result<()> {
    serialize_type(self, name)?;
    self.serialize_unit()
  }

  fn serialize_newtype_struct<T: ?Sized>(self, name: &'static str, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    use ser::SerializeTupleStruct;
    serialize_type(self, name)?;
    let mut s = self.serialize_struct(name, 1)?;
    s.serialize_field(value)?;
    s.end()
  }

  fn serialize_newtype_variant<T: ?Sized>(
    self,
    name: &'static str,
    variant_index: u32,
    variant: &'static str,
    value: &T,
  ) -> Result<()>
  where
    T: Serialize,
  {
    use ser::SerializeTupleVariant;
    serialize_type(self, variant)?;
    let mut tv = self.serialize_tuple_variant(name, variant_index, variant, 1)?;
    tv.serialize_field(value)?;
    tv.end()
  }

  fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
    self.output += "[";
    Ok(self)
  }

  fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
    self.serialize_seq(Some(len))
  }

  fn serialize_tuple_struct(
    self,
    name: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleStruct> {
    serialize_type(self, name)?;
    self.serialize_tuple(len)
  }

  fn serialize_tuple_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeTupleVariant> {
    self.serialize_tuple_struct(variant, len)
  }

  fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
    self.output += "{";
    Ok(self)
  }

  fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
    serialize_type(self, name)?;
    self.serialize_map(Some(len))
  }

  fn serialize_struct_variant(
    self,
    _name: &'static str,
    _variant_index: u32,
    variant: &'static str,
    len: usize,
  ) -> Result<Self::SerializeStructVariant> {
    self.serialize_struct(variant, len)
  }
}

impl<'a> ser::SerializeSeq for &'a mut Serializer {
  type Ok = ();

  type Error = Error;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    value.serialize(&mut **self)?;
    self.output += ",";
    Ok(())
  }

  fn end(self) -> Result<()> {
    self.output += "]";
    Ok(())
  }
}

impl<'a> ser::SerializeTuple for &'a mut Serializer {
  type Ok = ();

  type Error = Error;

  fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    ser::SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<()> {
    ser::SerializeSeq::end(self)
  }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    ser::SerializeTuple::serialize_element(self, value)
  }

  fn end(self) -> Result<()> {
    ser::SerializeTuple::end(self)
  }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    ser::SerializeSeq::serialize_element(self, value)
  }

  fn end(self) -> Result<()> {
    ser::SerializeSeq::end(self)
  }
}

impl<'a> ser::SerializeMap for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
  where
    T: Serialize,
  {
    key.serialize(&mut **self)?;
    self.output += " ";
    Ok(())
  }

  fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    value.serialize(&mut **self)?;
    self.output += ",";
    Ok(())
  }

  fn end(self) -> Result<()> {
    self.output += "}";
    Ok(())
  }
}

impl<'a> ser::SerializeStruct for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    ser::SerializeMap::serialize_entry(self, key, value)
  }

  fn end(self) -> Result<()> {
    ser::SerializeMap::end(self)
  }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
  type Ok = ();
  type Error = Error;

  fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
  where
    T: Serialize,
  {
    ser::SerializeMap::serialize_entry(self, key, value)
  }

  fn end(self) -> Result<()> {
    ser::SerializeMap::end(self)
  }
}
