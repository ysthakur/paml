use std::{iter::Peekable, str::CharIndices};

use crate::tree::Span;

pub struct Token {
  pub token_type: TokenType,
  pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
  Comma,
  /// `[`
  LSquare,
  /// `]`
  RSquare,
  /// `{`
  LBrace,
  /// `}`
  RBrace,
  /// `#[`
  MultilineCommentStart,
  /// `#]`
  MultilineCommentEnd,
  /// `#`
  SingleLineCommentStart,
  /// `\r`, `\n`, or `\r\n`
  Newline,
  /// Spaces and tabs
  HorizontalWhitespace,
  /// `<`
  Lt,
  /// `>`
  Gt,
  BareString,
  QuotedStringType,
  QuotedString {
    delimiter: String,
  },
}

pub enum TokenizeError {
  NoEndingQuote {
    /// The span of the start/opening/left quote
    start_span: Span,
  },
}

pub type TokenizeResult<T> = Result<T, TokenizeError>;

pub fn tokenize(text: &str) -> TokenizeResult<Vec<Token>> {
  let mut toks = Vec::new();

  let mut chars = text.char_indices().peekable();
  while let Some((ind, c)) = chars.next() {
    let mut add_tok = |tok_type: TokenType, byte_len: usize| {
      toks.push(Token { token_type: tok_type, span: Span { start: ind, end: ind + byte_len } });
    };
    match c {
      ',' => add_tok(TokenType::Comma, 1),
      '[' => add_tok(TokenType::LSquare, 1),
      ']' => add_tok(TokenType::RSquare, 1),
      '{' => add_tok(TokenType::LBrace, 1),
      '}' => add_tok(TokenType::RBrace, 1),
      '#' => match chars.peek() {
        Some((_, '[')) => {
          let _ = chars.next();
          add_tok(TokenType::MultilineCommentStart, 2);
        }
        Some((_, ']')) => {
          let _ = chars.next();
          add_tok(TokenType::MultilineCommentEnd, 2);
        }
        _ => add_tok(TokenType::SingleLineCommentStart, 1),
      },
      '\n' => add_tok(TokenType::Newline, 1),
      '\r' => match chars.peek() {
        Some((_, '\n')) => {
          let _ = chars.next();
          add_tok(TokenType::Newline, 2)
        }
        _ => add_tok(TokenType::SingleLineCommentStart, 1),
      },
      c if c.is_ascii_whitespace() => {
        let mut len = c.len_utf8();
        while let Some((_, next)) = chars.peek() {
          // &str always uses UTF-8 and these are ASCII characters anyway
          len += next.len_utf8();
          let _ = chars.next();
        }
        add_tok(TokenType::HorizontalWhitespace, len)
      }
      '\'' | '"' | '`' => {
        toks.push(string_token(c, &mut chars)?);
      }
      '<' => {
        todo!("parse type annotations")
      }
      _ => {
        let mut len = c.len_utf8();
        while let Some((_, next)) = chars.peek() {
          if is_special_char(*next) {
            break;
          }
          // &str always uses UTF-8
          len += next.len_utf8();
          let _ = chars.next();
        }
        match chars.peek() {
          Some((_, '\'' | '"')) => {
            add_tok(TokenType::QuotedStringType, len);
          }
          _ => add_tok(TokenType::BareString, len),
        }
      }
    };
  }

  Ok(toks)
}

fn is_special_char(c: char) -> bool {
  match c {
    ',' | '[' | ']' | '{' | '}' | '#' | '<' | '>' | '\'' | '"' | '`' => true,
    _ if c.is_ascii_whitespace() => true,
    _ => false,
  }
}

fn string_token(first_quote: char, chars: &mut Peekable<CharIndices>) -> TokenizeResult<Token> {
  let is_raw = first_quote == '`';
  // todo allow strings with multiple quotes
  let quote_char = first_quote;
  todo!()
}

mod test {
  // #[case]
  // fn tokenize_happy(#[case] text: &str, #[case] expected: Vec<Token>) {

  // }
}
