use std::iter::Peekable;

use crate::{
  tokenize::{Token, TokenType, TokenizeError, tokenize},
  tree::{Span, Tree, TreeInfo},
};

pub enum ParseError {
  EmptyFile,
  UnexpectedEof {
    expected: String,
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
}

pub type ParseResult<T> = Result<T, ParseError>;

pub fn parse(text: String) -> ParseResult<TreeInfo> {
  let tokens = tokenize(&text).map_err(|e| match e {
    TokenizeError::NoEndingQuote { start_span } => ParseError::UnmatchedStartDelimiter {
      expected: "ending quote".to_string(),
      cause_span: start_span,
    },
  })?;

  let mut parser = Parser {
    text,
    tokens: tokens.into_iter().peekable(),
  };

  let expr = parser.parse_expr("an expression")?;
  match parser.tokens.peek() {
    None => Ok(expr),
    Some(tok) => Err(ParseError::UnexpectedToken {
      span: tok.span.clone(),
    }),
  }
}

struct Parser<I>
where
  I: Iterator<Item = Token>,
{
  text: String,
  tokens: Peekable<I>,
}

impl<I> Parser<I>
where
  I: Iterator<Item = Token>,
{
  fn parse_expr(&mut self, expected: &str) -> ParseResult<Option<TreeInfo>> {
    self.skip_ignored_tokens()?;
    while let Some(tok) = self.tokens.peek() {
      match &tok.token_type {
        TokenType::LSquare => return self.parse_list().map(Some),
        TokenType::LBrace => return self.parse_map().map(Some),
        TokenType::Lt => return self.parse_type_annotated_value().map(Some),
        TokenType::BareString => {
          let span = tok.span;
          // todo detect numbers and booleans
          let tree = TreeInfo {
            tree: Tree::Str(self.get_span_contents(span).to_string()),
            typ: None,
            span,
          };
          return Ok(Some(tree));
        }
        TokenType::QuotedStringType => todo!(),
        TokenType::QuotedString { delimiter } => todo!(),
        TokenType::RSquare | TokenType::RBrace | TokenType::Gt | TokenType::MultilineCommentEnd => {
          let span = tok.span;
          return Err(ParseError::UnmatchedEndDelimiter {
            ending_delimiter: self.get_span_contents(span).to_string(),
            span,
          });
        }
        _ => return Err(ParseError::UnexpectedToken { span: tok.span }),
      }
    }

    Ok(None)
  }

  fn parse_list(&mut self) -> ParseResult<TreeInfo> {
    let start_tok = self
      .tokens
      .next()
      .expect("parse_list should only be called when there's a [");
    assert_eq!(start_tok.token_type, TokenType::LSquare);

    let mut items = Vec::new();
    loop {
      self.skip_ignored_tokens();
      match self.tokens.peek() {
        None => {
          return Err(ParseError::UnmatchedStartDelimiter {
            expected: "]".to_string(),
            cause_span: start_tok.span,
          });
        }
        Some(end_tok) if end_tok.token_type == TokenType::RSquare => {
          let span = Span {
            start: start_tok.span.start,
            end: end_tok.span.end,
          };
          let _ = self.tokens.next();
          return Ok(TreeInfo {
            tree: Tree::List(items),
            typ: None,
            span,
          });
        }
        Some(_) => {
          let Some(item) = self.parse_expr()? else {
            return Err(ParseE)
          };
          items.push(item);
        }
      }
    }
  }

  fn parse_map(&mut self) -> ParseResult<TreeInfo> {
    let start_tok = self
      .tokens
      .next()
      .expect("parse_map should only be called when there's a {");
    assert_eq!(start_tok.token_type, TokenType::LBrace);

    let mut items = Vec::new();
    loop {
      match self.tokens.peek() {
        None => {
          return Err(ParseError::UnmatchedStartDelimiter {
            expected: "}".to_string(),
            cause_span: start_tok.span,
          });
        }
        Some(end_tok) if end_tok.token_type == TokenType::RBrace => {
          let span = Span {
            start: start_tok.span.start,
            end: end_tok.span.end,
          };
          let _ = self.tokens.next();
          return Ok(TreeInfo {
            tree: Tree::List(items),
            typ: None,
            span,
          });
        }
        Some(_) => {
          let key = self.parse_expr()?;
          items.push(item);
        }
      }
    }
  }

  fn parse_type_annotated_value(&mut self) -> ParseResult<TreeInfo> {
    todo!()
  }

  fn parse_multiline_comment(&mut self) -> ParseResult<()> {
    let start_tok = self
      .tokens
      .next()
      .expect("parse_multiline_comment should only be called when there's a #[");
    assert_eq!(start_tok.token_type, TokenType::MultilineCommentStart);

    let mut start_stack = vec![start_tok.span];
    while let Some(tok) = self.tokens.next() {
      match tok.token_type {
        TokenType::MultilineCommentStart => {
          start_stack.push(tok.span);
        }
        TokenType::MultilineCommentEnd => {
          start_stack.pop();
        }
        _ => {}
      }
    }

    if let Some(last_span) = start_stack.pop() {
      Err(ParseError::UnmatchedStartDelimiter {
        expected: "#]".to_string(),
        cause_span: last_span,
      })
    } else {
      Ok(())
    }
  }

  fn parse_single_line_comment(&mut self) -> ParseResult<()> {
    todo!()
  }

  fn skip_ignored_tokens(&mut self) -> ParseResult<()> {
    while let Some(tok) = self.tokens.peek() {
      match tok.token_type {
        TokenType::Newline | TokenType::HorizontalWhitespace => {
          let _ = self.tokens.next();
        }
        TokenType::MultilineCommentStart => {
          let span = tok.span;
          let _ = self.tokens.next();
          self.parse_multiline_comment(span)?;
        }
        TokenType::SingleLineCommentStart => {
          let _ = self.tokens.next();
          self.parse_single_line_comment()?;
        }
        _ => break,
      }
    }

    Ok(())
  }

  fn get_span_contents(&self, span: Span) -> &str {
    &self.text[span.start..span.end]
  }
}
