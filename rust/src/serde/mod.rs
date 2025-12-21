mod de;
mod error;
mod ser;

pub use error::{Error, Result};
pub use de::{PamlDeserializer, from_str};
pub use ser::{Serializer, to_string};