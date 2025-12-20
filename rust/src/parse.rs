use std::iter::Peekable;

use crate::{
  Ast, Ignored, IgnoredKind, IgnoredPart, ListItem, MapItem, ParseError, ParseTree, Separator,
  Span, Token, TokenType, ValidationError, tokenize,
};

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Clone, Debug)]
pub struct LosslessParseResult {
  pub before: Ignored,
  pub tree: ParseTree,
  pub after: Ignored,
  validation_errors: Vec<ValidationError>,
}

impl LosslessParseResult {
  pub fn validation_errors(&self) -> Vec<ValidationError> {
    self.validation_errors.clone()
  }

  pub fn to_ast(self) -> std::result::Result<Ast, Vec<ValidationError>> {
    if self.validation_errors.is_empty() {
      Ok(tree_to_ast(self.tree))
    } else {
      Err(self.validation_errors)
    }
  }
}

fn tree_to_ast(tree: ParseTree) -> Ast {
  match tree {
    ParseTree::Bool { val, span } => Ast::Bool { val, span },
    ParseTree::Num { val, span } => Ast::Num { val, span },
    ParseTree::Str { val, span, delim_len: _ } => Ast::Str { val, span },
    ParseTree::List { opener, after_opener: _, items, closer } => Ast::List {
      val: items.into_iter().map(|it| tree_to_ast(it.item)).collect(),
      span: Span { start: opener.start, end: closer.end },
    },
    ParseTree::Map { opener, after_opener: _, items, closer } => Ast::List {
      val: items.into_iter().map(|it| tree_to_ast(it.key)).collect(),
      span: Span { start: opener.start, end: closer.end },
    },
  }
}

pub fn parse_lossless(text: String) -> Result<LosslessParseResult> {
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
    if let Some((text, delim_len, span)) = self.parse_quoted_string() {
      Ok(Some(ParseTree::Str { val: text, delim_len, span }))
    } else if let Some(tok) = self.consume_if(|tok| tok.token_type == TokenType::BareString) {
      if let Some((text, delim_len, str_span)) = self.parse_quoted_string() {
        // This is a string with a formatting type
        Ok(Some(ParseTree::Str {
          val: text, // TODO change the text according to the format type
          delim_len,
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
            delim_len: 0,
            span: tok.span,
          }))
        }
      }
    } else {
      Ok(None)
    }
  }

  fn parse_quoted_string(&mut self) -> Option<(String, usize, Span)> {
    match self.tokens.peek() {
      Some(Token { token_type: TokenType::QuotedString { delim_len }, span }) => {
        let delim_len = *delim_len;
        let span = *span;
        let content = self.get_span_contents(span);
        let text = content[delim_len..content.len() - delim_len].to_string();
        let _ = self.tokens.next();
        Some((text, delim_len, span))
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
        return Ok(Some(ParseTree::List {
          opener: start_tok.span,
          after_opener,
          items,
          closer: end_tok.span,
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
        return Ok(Some(ParseTree::Map {
          opener: start_tok.span,
          after_opener,
          items,
          closer: end_tok.span,
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
