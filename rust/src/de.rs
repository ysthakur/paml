use serde::de::{self, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{forward_to_deserialize_any, Deserialize};

use crate::error::{Error, Result};

pub struct Deserializer<'de> {
    // This string starts with the input data and characters are truncated off
    // the beginning as data is parsed.
    input: &'de str,
}

impl<'de> Deserializer<'de> {
    pub fn from_str(input: &'de str) -> Self {
        Deserializer { input }
    }
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);
    let t = T::deserialize(&mut deserializer)?;
    if deserializer.input.is_empty() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters(deserializer.input.to_string()))
    }
}

const SPECIAL_CHARS: [char; 4] = ['{', '}', '[', ']'];

impl<'de> Deserializer<'de> {
    fn peek(&mut self) -> Result<char> {
        self.input.chars().next().ok_or(Error::Eof)
    }

    fn next(&mut self) -> Result<char> {
        let c = self.peek()?;
        self.input = &self.input[c.len_utf8()..];
        Ok(c)
    }

    fn trim_whitespace(&mut self) -> Result<()> {
        todo!()
    }

    fn parse_keyword(&mut self, keyword: &str) -> Result<bool> {
        if !self.input.starts_with(keyword) {
            Ok(false)
        } else if self.input.len() == keyword.len() {
            Ok(true) // EOF right afterwards
        } else {
            let e = self.input.chars().nth(keyword.len() + 1).unwrap();
            if e.is_whitespace() || SPECIAL_CHARS.contains(&e) {
                self.input = &self.input[keyword.len()..];
                Ok(true)
            } else {
                Ok(false)
            }
        }
    }

    fn parse_type(&mut self) -> Result<Option<String>> {
        if self.input.starts_with("~") {
            let _ = self.next()?;
            let word = self.parse_str()?;
            self.trim_whitespace()?;
            Ok(Some(word))
        } else {
            Ok(None)
        }
    }

    fn parse_str(&mut self) -> Result<String> {
        match self.peek()? {
            q @ ('"' | '\'') => {
                // Normal quoted strings
                // todo allow raw strings with r#""#
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
                    Ok(str)
                }
            }
            _ => {
                // Bare strings (single words)
                let word: String = self.input.chars().take_while(|c| !c.is_whitespace()).collect();
                if word.is_empty() {
                    Err(Error::Message(
                        "Expected a word, got whitespace".to_string(),
                    ))
                } else {
                    self.input = &self.input[word.len()..];
                    Ok(word)
                }
            }
        }
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.trim_whitespace()?;
        if self.input.is_empty() {
            Err(Error::Eof)
        } else {
            let typ = self.parse_type()?;

            if self.parse_keyword("true")? {
                visitor.visit_bool(true)
            } else if self.parse_keyword("false")? {
                visitor.visit_bool(false)
            } else if self.parse_keyword("null")? {
                if let Some("None") = typ.as_deref() {
                    visitor.visit_none()
                } else {
                    visitor.visit_unit()
                }
            } else if self.input.starts_with("[") {
                if let Some("Some") = typ.as_deref() {
                    visitor.visit_enum(self)
                } else {
                    visitor.visit_seq(self)
                }
            } else if self.input.starts_with("{") {
                visitor.visit_map(self)
            } else {
                visitor.visit_string(self.parse_str()?)
            }
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple map identifier
        struct tuple_struct ignored_any
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let typ = self.parse_type()?;
    }
}

impl<'de, 'a> SeqAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.trim_whitespace()?;
        if self.peek()? == ']' {
            return Ok(None);
        } else {
            seed.deserialize(&mut **self).map(Some)
        }
    }
}

impl<'de, 'a> MapAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        self.trim_whitespace()?;
        if self.peek()? == '}' {
            return Ok(None);
        } else {
            seed.deserialize(&mut **self).map(Some)
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.trim_whitespace()?;
        if self.peek()? == '}' {
            return Err(Error::Message("No value given".to_string()));
        } else {
            seed.deserialize(&mut **self)
        }
    }
}

impl<'de, 'a> EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: de::DeserializeSeed<'de>,
    {
        todo!()
    }
}

impl<'de, 'a> VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        todo!()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        todo!()
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }
}
