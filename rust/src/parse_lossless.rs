use std::iter::Peekable;

use crate::{
  tokenize::{Token, TokenType, TokenizeError, tokenize},
  tree::{Num, Span},
};

#[derive(Debug)]
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
    span: Span,
  },
  List {
    opener: Span,
    after_opener: Ignored,
    items: Vec<ListItem>,
    closer: Span,
    after_closer: Ignored,
  },
  Map {
    opener: Span,
    after_opener: Ignored,
    items: Vec<MapItem>,
    closer: Span,
    after_closer: Ignored,
  },
}

impl ParseTree {
  fn span(&self) -> Span {
    match self {
      ParseTree::Bool { span, .. } => *span,
      ParseTree::Num { span, .. } => *span,
      ParseTree::Str { span, .. } => *span,
      ParseTree::List { opener, closer, after_closer, .. } => {
        let end = after_closer.parts.last().map(|part| part.span.end).unwrap_or(closer.end);
        Span { start: opener.start, end }
      }
      ParseTree::Map { opener, closer, after_closer, .. } => {
        let end = after_closer.parts.last().map(|part| part.span.end).unwrap_or(closer.end);
        Span { start: opener.start, end }
      }
    }
  }
}

#[derive(Debug)]
pub struct ListItem {
  pub item: ParseTree,
  pub after_item: Ignored,
  /// The comma after this list item
  pub sep: Option<Separator>,
}

#[derive(Debug)]
pub struct MapItem {
  pub key: ParseTree,
  pub after_key: Ignored,
  pub val: ParseTree,
  pub after_val: Ignored,
  /// The comma or newline after this map item
  pub sep: Option<Separator>,
}

/// Span for a comma
#[derive(Debug)]
pub struct Separator {
  pub sep: Span,
  /// The ignored whitespace/comments after the separator
  pub after: Ignored,
}

/// Holds spans for whitespace and comments
#[derive(Debug)]
pub struct Ignored {
  pub parts: Vec<IgnoredPart>,
}

#[derive(Debug)]
pub struct IgnoredPart {
  pub span: Span,
  pub kind: IgnoredKind,
}

#[derive(Debug)]
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

pub type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug)]
pub enum ValidationError {
  DuplicateKey { key: String, orig_span: Span, dupe_span: Span },
  UnrecognizedStringFormatType { span: Span },
}

pub struct LosslessParseResult {
  pub before: Ignored,
  pub tree: ParseTree,
  pub after: Ignored,
  pub validation_errors: Vec<ValidationError>,
}

pub fn parse(text: String) -> Result<LosslessParseResult> {
  let tokens = tokenize(&text).map_err(|err| ParseError::TokenizeError { err })?;

  let mut parser =
    Parser { text, tokens: tokens.into_iter().peekable(), validation_errors: Vec::new() };

  let before = parser.parse_ignored()?;
  let expr = parser.parse_expr()?;
  let after = parser.parse_ignored()?;

  match (expr, parser.tokens.peek()) {
    (Some(expr), None) => Ok(LosslessParseResult {
      before,
      tree: expr,
      after,
      validation_errors: parser.validation_errors,
    }),
    (None, None) => Err(ParseError::EmptyFile),
    (_, Some(tok)) => Err(ParseError::UnexpectedToken { span: tok.span.clone() }),
  }
}

struct Parser<I>
where
  I: Iterator<Item = Token>,
{
  text: String,
  tokens: Peekable<I>,
  validation_errors: Vec<ValidationError>,
}

impl<I> Parser<I>
where
  I: Iterator<Item = Token>,
{
  fn parse_expr(&mut self) -> Result<Option<ParseTree>> {
    if let Some(tree) = self.parse_string()? {
      return Ok(Some(tree));
    }
    if let Some(tree) = self.parse_list()? {
      return Ok(Some(tree));
    }
    if let Some(tree) = self.parse_map()? {
      return Ok(Some(tree));
    }

    Ok(None)
  }

  fn parse_string(&mut self) -> Result<Option<ParseTree>> {
    if let Some((text, span)) = self.parse_quoted_string() {
      Ok(Some(ParseTree::Str { val: text, span }))
    } else if let Some(tok) = self.consume_if(|tok| tok.token_type == TokenType::BareString) {
      if let Some((text, str_span)) = self.parse_quoted_string() {
        // This is a string with a formatting type
        Ok(Some(ParseTree::Str {
          val: text, // TODO change the text according to the format type
          span: Span { start: tok.span.start, end: str_span.end },
        }))
      } else {
        // This is just a bare word
        let contents = self.get_span_contents(tok.span);
        if contents == "true" {
          Ok(Some(ParseTree::Bool { val: true, span: tok.span }))
        } else if contents == "false" {
          Ok(Some(ParseTree::Bool { val: false, span: tok.span }))
        } else {
          // todo detect numbers
          Ok(Some(ParseTree::Str {
            val: self.get_span_contents(tok.span).to_string(),
            span: tok.span,
          }))
        }
      }
    } else {
      Ok(None)
    }
  }

  fn parse_quoted_string(&mut self) -> Option<(String, Span)> {
    match self.tokens.peek() {
      Some(Token { token_type: TokenType::QuotedString { delim_len }, span }) => {
        let delim_len = *delim_len;
        let span = *span;
        let content = self.get_span_contents(span);
        let text = content[delim_len..content.len() - delim_len].to_string();
        let _ = self.tokens.next();
        Some((text, span))
      }
      _ => None,
    }
  }

  fn parse_list(&mut self) -> Result<Option<ParseTree>> {
    let Some(start_tok) = self.consume_if(|tok| tok.token_type == TokenType::LSquare) else {
      return Ok(None);
    };
    let after_opener = self.parse_ignored()?;

    let mut items = Vec::new();
    loop {
      if let Some(item) = self.parse_expr()? {
        let after_item = self.parse_ignored()?;
        let sep = self.parse_item_sep()?;
        items.push(ListItem { item, after_item, sep })
      } else if let Some(end_tok) = self.consume_if(|tok| tok.token_type == TokenType::RSquare) {
        let after_closer = self.parse_ignored()?;
        return Ok(Some(ParseTree::List {
          opener: start_tok.span,
          after_opener,
          items,
          closer: end_tok.span,
          after_closer,
        }));
      } else {
        return Err(ParseError::UnmatchedStartDelimiter {
          expected: "]".to_string(),
          cause_span: start_tok.span,
        });
      }
    }
  }

  fn parse_map(&mut self) -> Result<Option<ParseTree>> {
    let Some(start_tok) = self.consume_if(|tok| tok.token_type == TokenType::LBrace) else {
      return Ok(None);
    };
    let after_opener = self.parse_ignored()?;

    let mut items = Vec::new();
    loop {
      if let Some(key) = self.parse_expr()? {
        let after_key = self.parse_ignored()?;
        let Some(val) = self.parse_expr()? else {
          return Err(self.expected_value_error("", key.span()));
        };
        let after_val = self.parse_ignored()?;
        let sep = self.parse_item_sep()?;
        items.push(MapItem { key, after_key, val, after_val, sep })
      } else if let Some(end_tok) = self.consume_if(|tok| tok.token_type == TokenType::RSquare) {
        let after_closer = self.parse_ignored()?;
        return Ok(Some(ParseTree::Map {
          opener: start_tok.span,
          after_opener,
          items,
          closer: end_tok.span,
          after_closer,
        }));
      } else {
        return Err(ParseError::UnmatchedStartDelimiter {
          expected: "]".to_string(),
          cause_span: start_tok.span,
        });
      }
    }
  }

  fn expected_value_error(&mut self, msg: &str, cause_span: Span) -> ParseError {
    if let Some(tok) = self.tokens.peek() {
      ParseError::ExpectedValue { msg: msg.to_string(), span: tok.span }
    } else {
      ParseError::UnexpectedEof { expected: msg.to_string(), cause_span }
    }
  }

  /// Consume and return the next token if it matches the given predicate
  fn consume_if(&mut self, pred: impl FnOnce(&Token) -> bool) -> Option<Token> {
    let matches = self.tokens.peek().map(pred).unwrap_or(false);
    if matches {
      Some(self.tokens.next().expect("there should be a token if matches is true"))
    } else {
      None
    }
  }

  /// Parse a list/map item separator (comma)
  fn parse_item_sep(&mut self) -> Result<Option<Separator>> {
    if let Some(comma) = self.consume_if(|tok| tok.token_type == TokenType::Comma) {
      let after = self.parse_ignored()?;
      Ok(Some(Separator { sep: comma.span, after }))
    } else {
      Ok(None)
    }
  }

  /// Consume whitespace and comments
  ///
  /// Parameters:
  /// * `include_newline` - If `false`, stops before consuming a newline token
  ///   (newlines after single-line comments won't be consumed either)
  fn parse_ignored(&mut self) -> Result<Ignored> {
    let mut parts = Vec::new();
    loop {
      let num_parts_start = parts.len();
      if let Some(horiz_ws) = self.parse_horizontal_whitespace() {
        parts.push(horiz_ws);
      }
      if let Some(line_comment) = self.parse_single_line_comment() {
        parts.push(line_comment);
      }
      if let Some(multi_line_comment) = self.parse_multiline_comment()? {
        parts.push(multi_line_comment);
      }
      if let Some(newline) = self.consume_if(|tok| tok.token_type == TokenType::Newline) {
        parts.push(IgnoredPart { span: newline.span, kind: IgnoredKind::Newline });
      }

      let added_new = parts.len() > num_parts_start;
      if !added_new {
        break;
      }
    }

    Ok(Ignored { parts })
  }

  fn parse_horizontal_whitespace(&mut self) -> Option<IgnoredPart> {
    let Some(first) = self.consume_if(|tok| tok.token_type == TokenType::HorizontalWhitespace)
    else {
      return None;
    };
    let mut end = first.span.end;
    while let Some(next) = self.consume_if(|tok| tok.token_type == TokenType::HorizontalWhitespace)
    {
      end = next.span.end;
    }
    Some(IgnoredPart {
      span: Span { start: first.span.start, end },
      kind: IgnoredKind::HorizontalWhitespace,
    })
  }

  fn parse_single_line_comment(&mut self) -> Option<IgnoredPart> {
    let Some(start_tok) =
      self.consume_if(|tok| tok.token_type == TokenType::SingleLineCommentStart)
    else {
      return None;
    };
    let mut end = start_tok.span.end;
    while let Some(next) = self.consume_if(|tok| tok.token_type != TokenType::Newline) {
      end = next.span.end;
    }

    Some(IgnoredPart {
      span: Span { start: start_tok.span.start, end },
      kind: IgnoredKind::SingleLineComment,
    })
  }

  fn parse_multiline_comment(&mut self) -> Result<Option<IgnoredPart>> {
    let Some(start_tok) = self.consume_if(|tok| tok.token_type == TokenType::MultilineCommentStart)
    else {
      return Ok(None);
    };

    let mut start_stack = vec![start_tok.span];
    while let Some(tok) = self.tokens.next() {
      match tok.token_type {
        TokenType::MultilineCommentStart => {
          start_stack.push(tok.span);
        }
        TokenType::MultilineCommentEnd => {
          if let Some(start_span) = start_stack.pop() {
            if start_stack.is_empty() {
              return Ok(Some(IgnoredPart {
                span: Span { start: start_span.start, end: tok.span.end },
                kind: IgnoredKind::MultilineComment,
              }));
            }
          } else {
            return Err(ParseError::UnmatchedEndDelimiter {
              ending_delimiter: "#]".to_string(),
              span: tok.span,
            });
          }
        }
        _ => {}
      }
    }

    let last_span = start_stack
      .pop()
      .expect("stack cannot be empty because after popping, we return if it's empty");
    Err(ParseError::UnmatchedStartDelimiter { expected: "#]".to_string(), cause_span: last_span })
  }

  fn get_span_contents(&self, span: Span) -> &str {
    &self.text[span.start..span.end]
  }
}
