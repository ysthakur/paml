mod parse;
mod print;
pub mod serde;
mod tokenize;

use std::collections::HashMap;
use std::hash::Hash;

pub use parse::{LosslessParseResult, parse_lossless};
pub use print::print;
pub use tokenize::{Token, TokenType, TokenizeError, tokenize};

/// The start and offset of a [Tree]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug)]
pub enum Value {
  Bool { val: bool, span: Span },
  Num { val: Num, span: Span },
  Str { val: String, span: Span },
  List { val: Vec<Value>, span: Span },
  Map { val: HashMap<Value, Value>, span: Span },
}

impl PartialEq for Value {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Self::Bool { val: l_val, .. }, Self::Bool { val: r_val, .. }) => l_val == r_val,
      (Self::Num { val: l_val, .. }, Self::Num { val: r_val, .. }) => l_val == r_val,
      (Self::Str { val: l_val, .. }, Self::Str { val: r_val, .. }) => l_val == r_val,
      (Self::List { val: l_val, .. }, Self::List { val: r_val, .. }) => l_val == r_val,
      (Self::Map { val: l_val, .. }, Self::Map { val: r_val, .. }) => l_val == r_val,
      _ => false,
    }
  }
}

impl Eq for Value {}

impl Hash for Value {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    match self {
      Value::Bool { val, .. } => val.hash(state),
      Value::Num { val, .. } => val.hash(state),
      Value::Str { val, .. } => val.hash(state),
      Value::List { val, .. } => val.hash(state),
      Value::Map { .. } => {
        // Don't do anything, so that HashMaps all have the same hash
        // TODO this is probably a bad idea, possibly hash the *sorted* keyset?
      }
    }
  }
}

// TODO properly implemlent PartialEq and Hash
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Num {
  pub integer_part: String,
  pub decimal_part: Option<String>,
  pub exponent: Option<String>,
}

#[derive(Clone, Debug)]
pub enum QuotedStringType {
  /// Unindent the string to the specified level
  Unindent,
  /// Replace all line breaks with spaces, turning the string into a single line
  SingleLine,
}

impl QuotedStringType {
  /// Parse a [QuotedStringType] from a string. Returns [Option] if the type
  /// isn't recognized.
  ///
  /// I didn't feel like implementing [std::str::FromStr] because there won't be
  /// any meaningful errors.
  pub fn from_str(s: &str) -> Option<QuotedStringType> {
    match s {
      "unindent" => Some(QuotedStringType::Unindent),
      "singleLine" => Some(QuotedStringType::SingleLine),
      _ => None,
    }
  }
}

#[derive(Clone, Debug)]
pub enum ParseTree {
  Bool {
    val: bool,
    span: Span,
  },
  Num {
    val: Num,
    span: Span,
  },
  BareString {
    val: String,
    span: Span,
  },
  QuotedString {
    val: String,
    string_type: Option<QuotedStringType>,
    /// Length of the delimiter in bytes (e.g. """foo""" would have `delim_len` 3).
    /// Must be an odd number.
    delim_len: usize,
    span: Span,
  },
  List {
    opener: Span,
    after_opener: Ignored,
    items: Vec<ListItem>,
    closer: Span,
  },
  Map {
    opener: Span,
    after_opener: Ignored,
    items: Vec<MapItem>,
    closer: Span,
  },
}

impl ParseTree {
  fn span(&self) -> Span {
    match self {
      ParseTree::Bool { span, .. } => *span,
      ParseTree::Num { span, .. } => *span,
      ParseTree::BareString { span, .. } => *span,
      ParseTree::QuotedString { span, .. } => *span,
      ParseTree::List { opener, closer, .. } => Span { start: opener.start, end: closer.end },
      ParseTree::Map { opener, closer, .. } => Span { start: opener.start, end: closer.end },
    }
  }
}

#[derive(Clone, Debug)]
pub struct ListItem {
  pub item: ParseTree,
  pub after_item: Ignored,
  /// The comma after this list item
  pub sep: Option<Separator>,
}

#[derive(Clone, Debug)]
pub struct MapItem {
  pub key: ParseTree,
  pub after_key: Ignored,
  pub val: ParseTree,
  pub after_val: Ignored,
  /// The comma or newline after this map item
  pub sep: Option<Separator>,
}

/// Span for a comma
#[derive(Clone, Debug)]
pub struct Separator {
  pub sep: Span,
  /// The ignored whitespace/comments after the separator
  pub after: Ignored,
}

/// Holds spans for whitespace and comments
#[derive(Clone, Debug)]
pub struct Ignored {
  pub parts: Vec<IgnoredPart>,
}

#[derive(Clone, Debug)]
pub struct IgnoredPart {
  pub span: Span,
  pub kind: IgnoredKind,
}

#[derive(Clone, Debug)]
pub enum IgnoredKind {
  HorizontalWhitespace,
  Newline,
  /// This does not include the newline at the end of the comment (if any)
  SingleLineComment,
  MultilineComment,
}

#[derive(Debug)]
pub enum ParseError {
  EmptyFile,
  ExpectedValue {
    msg: String,
    span: Span,
  },
  UnexpectedEof {
    expected: String,
    cause_span: Span,
  },
  UnrecognizedStringType {
    span: Span,
  },
  /// Hit EOF before finding the matching end delimiter
  UnmatchedStartDelimiter {
    expected: String,
    cause_span: Span,
  },
  UnmatchedEndDelimiter {
    ending_delimiter: String,
    span: Span,
  },
  UnexpectedToken {
    span: Span,
  },
  TokenizeError {
    err: TokenizeError,
  },
}

#[derive(Clone, Debug)]
pub enum ValidationError {
  DuplicateKey { key: String, orig_span: Span, dupe_span: Span },
  UnrecognizedStringFormatType { span: Span },
}
