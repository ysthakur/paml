mod de;
mod error;
// pub mod parse;
pub mod parse_lossless;
mod ser;
pub mod tokenize;
pub mod tree;

pub use de::{from_str, PamlDeserializer};
pub use error::{Error, Result};
pub use ser::{to_string, Serializer};

pub fn add(left: usize, right: usize) -> usize {
  left + right
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let result = add(2, 2);
    assert_eq!(result, 4);
  }
}
