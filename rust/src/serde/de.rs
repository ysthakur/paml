use serde::de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{Deserialize, forward_to_deserialize_any};

use crate::serde::error::{Error, Result};

pub struct PamlDeserializer<'de> {
  // This string starts with the input data and characters are truncated off
  // the beginning as data is parsed.
  input: &'de str,
}

impl<'de> PamlDeserializer<'de> {
  pub fn from_str(input: &'de str) -> Self {
    PamlDeserializer { input }
  }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
  T: Deserialize<'a>,
{
  let mut deserializer = PamlDeserializer::from_str(s);
  let t = T::deserialize(&mut deserializer)?;
  if deserializer.input.is_empty() {
    Ok(t)
  } else {
    Err(Error::TrailingCharacters(deserializer.input.to_string()))
  }
}

const SPECIAL_CHARS: [char; 4] = ['{', '}', '[', ']'];

impl<'de> PamlDeserializer<'de> {
  fn peek(&mut self) -> Result<char> {
    self.input.chars().next().ok_or(Error::Eof)
  }

  fn next(&mut self) -> Result<char> {
    let c = self.peek()?;
    self.input = &self.input[c.len_utf8()..];
    Ok(c)
  }

  /// Whether the given character marks a word boundary
  fn ends_word(c: char) -> bool {
    SPECIAL_CHARS.contains(&c) || c.is_whitespace()
  }

  fn trim_ignored(&mut self) -> Result<()> {
    while !self.input.is_empty() {
      let c = self.peek()?;
      if c.is_whitespace() {
        let ws: String = self.input.chars().take_while(|c| c.is_whitespace()).collect();
        self.input = &self.input[ws.len()..];
      } else if c == '#' {
      } else {
        break;
      }
    }
    Ok(())
  }

  fn parse_keyword(&mut self, keyword: &str) -> Result<bool> {
    if !self.input.starts_with(keyword) {
      Ok(false)
    } else {
      let e = self.input.chars().nth(keyword.len());
      if e.is_none() || Self::ends_word(e.unwrap()) {
        self.input = &self.input[keyword.len()..];
        Ok(true)
      } else {
        Ok(false)
      }
    }
  }

  fn parse_str(&mut self) -> Result<String> {
    match self.peek()? {
      q @ ('"' | '\'') => {
        // Normal quoted strings
        // todo allow raw strings with r#""#
        self.next()?;
        let mut res = String::new();
        while !self.input.is_empty() {
          let c = self.next()?;
          if c == q {
            break;
          } else if c == '\\' {
            res.push(self.next()?);
          } else {
            res.push(c);
          }
        }
        Ok(res)
      }
      '`' => {
        // Strings that extend to the end of the line
        let str: String = self.input.chars().take_while(|&c| c != '\n').collect();
        if str.is_empty() {
          Err(Error::Message("Expected a string, got nothing".to_string()))
        } else {
          self.input = &self.input[str.len()..];
          Ok(str)
        }
      }
      _ => {
        // Bare strings (single words)
        let word: String = self.input.chars().take_while(|&c| !Self::ends_word(c)).collect();
        if word.is_empty() {
          Err(Error::Message("Expected a word, got whitespace".to_string()))
        } else {
          self.input = &self.input[word.len()..];
          Ok(word)
        }
      }
    }
  }

  fn parse_num(&mut self) -> Result<Option<String>> {
    // todo handle floats
    let num: String = self.input.chars().take_while(|c| c.is_digit(10)).collect();
    if !num.is_empty()
      && (self.input.is_empty() || Self::ends_word(self.input.chars().nth(num.len()).unwrap()))
    {
      self.input = &self.input[num.len()..];
      Ok(Some(num))
    } else {
      Ok(None)
    }
  }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut PamlDeserializer<'de> {
  type Error = Error;

  fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.trim_ignored()?;
    if self.input.is_empty() {
      Err(Error::Eof)
    } else {
      let c = self.peek()?;

      if self.parse_keyword("true")? {
        visitor.visit_bool(true)
      } else if self.parse_keyword("false")? {
        visitor.visit_bool(false)
      } else if self.parse_keyword("null")? {
        visitor.visit_unit()
      } else if c == '[' {
        self.next()?;
        visitor.visit_seq(self)
      } else if c == '{' {
        self.next()?;
        visitor.visit_map(self)
      } else {
        match self.parse_num()? {
          Some(num) => visitor.visit_i32(num.parse().unwrap()),
          None => visitor.visit_string(self.parse_str()?),
        }
      }
    }
  }

  forward_to_deserialize_any! {
      bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char string
      bytes byte_buf option unit unit_struct seq map
      struct tuple_struct ignored_any
  }

  fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let val = self.deserialize_seq(visitor)?;
    if self.next()? != ']' { Err(Error::Message("Expected ']'".to_string())) } else { Ok(val) }
  }

  fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.trim_ignored()?;
    if self.next()? != '~' { Err(Error::ExpectedType) } else { visitor.visit_newtype_struct(self) }
  }

  fn deserialize_enum<V>(
    self,
    _name: &'static str,
    _variants: &'static [&'static str],
    visitor: V,
  ) -> std::result::Result<V::Value, Self::Error>
  where
    V: Visitor<'de>,
  {
    self.trim_ignored()?;
    if self.next()? != '~' { Err(Error::ExpectedType) } else { visitor.visit_enum(self) }
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    self.trim_ignored()?;
    self.deserialize_str(visitor)
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    visitor.visit_string(self.parse_str()?)
  }
}

impl<'de, 'a> SeqAccess<'de> for &'a mut PamlDeserializer<'de> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
    T: de::DeserializeSeed<'de>,
  {
    self.trim_ignored()?;
    if self.peek()? == ']' {
      self.next()?;
      Ok(None)
    } else {
      seed.deserialize(&mut **self).map(Some)
    }
  }
}

impl<'de, 'a> MapAccess<'de> for &'a mut PamlDeserializer<'de> {
  type Error = Error;

  fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
  where
    K: de::DeserializeSeed<'de>,
  {
    self.trim_ignored()?;
    if self.peek()? == '}' {
      self.next()?;
      Ok(None)
    } else {
      seed.deserialize(&mut **self).map(Some)
    }
  }

  fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
  where
    V: de::DeserializeSeed<'de>,
  {
    self.trim_ignored()?;
    if self.peek()? == '}' {
      return Err(Error::Message("No value given".to_string()));
    } else {
      seed.deserialize(&mut **self)
    }
  }
}

impl<'de, 'a> EnumAccess<'de> for &'a mut PamlDeserializer<'de> {
  type Error = Error;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
  where
    V: de::DeserializeSeed<'de>,
  {
    let val = seed.deserialize(&mut *self)?;
    self.trim_ignored()?;
    Ok((val, self))
  }
}

impl<'de, 'a> VariantAccess<'de> for &'a mut PamlDeserializer<'de> {
  type Error = Error;

  fn unit_variant(self) -> Result<()> {
    self.trim_ignored()?;
    if self.parse_keyword("null")? {
      Ok(())
    } else {
      Err(Error::Message("Expected 'null'".to_string()))
    }
  }

  fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
  where
    T: de::DeserializeSeed<'de>,
  {
    seed.deserialize(self)
  }

  fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    let val = de::Deserializer::deserialize_seq(&mut *self, visitor)?;
    self.trim_ignored()?;
    if self.next()? != ']' { Err(Error::Message("Expected ']'".to_string())) } else { Ok(val) }
  }

  fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
  where
    V: Visitor<'de>,
  {
    de::Deserializer::deserialize_map(&mut *self, visitor)
  }
}

#[cfg(test)]
mod test {
  use serde::Deserialize;

  #[derive(Deserialize, PartialEq, Debug)]
  enum Enum {
    UnitVariant,
    NewTypeVariant(bool),
    TupleVariant(String, i32),
    StructVariant { null: (), foo: String },
  }

  #[derive(Deserialize, PartialEq, Debug)]
  struct Struct {
    seq: Vec<i32>,
  }

  #[test]
  fn test_literals() {
    assert_eq!((), super::from_str("null").unwrap());
    assert_eq!(true, super::from_str("true").unwrap());
    assert_eq!(false, super::from_str("false").unwrap());
    assert_eq!("123a", super::from_str::<String>("123a").unwrap());
  }

  #[test]
  fn test_seq() {
    let paml = "{ seq [0 1 2] }";
    assert_eq!(Struct { seq: vec![0, 1, 2] }, super::from_str(paml).unwrap());
  }

  #[test]
  fn test_tuple() {
    let paml = "[0 1 2]";
    assert_eq!((0, 1, 2), super::from_str(paml).unwrap());
  }

  #[test]
  fn test_enum() {
    let paml = "~UnitVariant null";
    assert_eq!(Enum::UnitVariant, super::from_str(paml).unwrap());

    let paml = "~NewTypeVariant true";
    assert_eq!(Enum::NewTypeVariant(true), super::from_str(paml).unwrap());

    let paml = r#"~TupleVariant ["foo" 45]"#;
    assert_eq!(Enum::TupleVariant("foo".to_string(), 45), super::from_str(paml).unwrap());

    let paml = r#"~StructVariant { null null foo bar }"#;
    assert_eq!(
      Enum::StructVariant { null: (), foo: "bar".to_string() },
      super::from_str(paml).unwrap()
    );
  }
}
