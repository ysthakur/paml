mod de;
mod error;
mod parse;
mod ser;
mod tokenize;

pub use de::{PamlDeserializer, from_str};
pub use error::{Error, Result};
pub use parse::{LosslessParseResult, parse_lossless};
pub use ser::{Serializer, to_string};
pub use tokenize::{Token, TokenType, TokenizeError, tokenize};

/// The start and offset of a [Tree]
#[derive(Clone, Copy, Debug)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

#[derive(Debug)]
pub enum Ast {
  Bool { val: bool, span: Span },
  Num { val: Num, span: Span },
  Str { val: String, span: Span },
  List { val: Vec<Ast>, span: Span },
  Map { val: Vec<(Ast, Ast)>, span: Span },
}

#[derive(Clone, Debug)]
pub struct Num {
  pub integer_part: String,
  pub decimal_part: String,
  pub exponent: String,
}

pub enum QuotedStringType {
  Unindent,
  SingleLine,
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
  Str {
    val: String,
    /// Length of the delimiter in bytes (e.g. """foo""" would have `delim_len` 3).
    /// This is 0 for barewords.
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
      ParseTree::Str { span, .. } => *span,
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
