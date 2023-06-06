

use serde::{Deserialize, forward_to_deserialize_any};
use serde::de::{self, Visitor};

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
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: Visitor<'de> {
        self.trim_whitespace()?;
        if self.input.is_empty() {
            Err(Error::Eof)
        } else if self.parse_keyword("true")? {
            visitor.visit_bool(true)
        } else if self.parse_keyword("false")? {
            visitor.visit_bool(false)
        } else if self.parse_keyword("null")? {
            visitor.visit_unit()
        } else {
            todo!()
        }
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
