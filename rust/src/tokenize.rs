use std::{cmp::Ordering, iter::Peekable, str::CharIndices};

use crate::tree::Span;

#[derive(Debug)]
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
  QuotedString {
    /// The length of the delimiter in bytes (when UTF-8 encoded)
    delim_len: usize,
  },
}

#[derive(Debug)]
pub enum TokenizeError {
  /// EOF hit before the ending quote of a string was reached
  NoEndingQuote {
    /// The span of the start/opening/left quote
    open_span: Span,
  },
  /// EOF hit right after a backslash in a string
  NoEscapedCharacter { span: Span },
  /// n consecutive quotes encountered, where n is a multiple of 4
  IncorrectOpeningQuotes { span: Span },
  /// The string ends with more quote characters than it starts with, e.g., `"foo"""`
  MismatchedEndingQuotes {
    /// The span of the opening quotes
    open_span: Span,
    /// The span of the ending quotes
    end_span: Span,
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
          if !next.is_ascii_whitespace() {
            break;
          }
          // &str always uses UTF-8 and these are ASCII characters anyway
          len += next.len_utf8();
          let _ = chars.next();
        }
        add_tok(TokenType::HorizontalWhitespace, len)
      }
      '\'' | '"' | '`' => {
        toks.push(string_token(c, ind, &mut chars)?);
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
        add_tok(TokenType::BareString, len);
      }
    };
  }

  Ok(toks)
}

fn is_special_char(c: char) -> bool {
  match c {
    ',' | '[' | ']' | '{' | '}' | '#' | '\'' | '"' | '`' => true,
    _ if c.is_ascii_whitespace() => true,
    _ => false,
  }
}

fn string_token(
  quote_char: char,
  start: usize,
  chars: &mut Peekable<CharIndices>,
) -> TokenizeResult<Token> {
  let is_raw = quote_char == '`';
  let quote_len = quote_char.len_utf8();

  let mut end = start + quote_len;
  let mut num_quotes = 1;
  loop {
    match chars.peek() {
      Some((_, c)) if *c == quote_char => {
        chars.next();
        end += quote_len;
        num_quotes += 1;
      }
      _ => break,
    }
  }

  // This is an empty string
  if num_quotes % 2 == 0 {
    // Since this is an empty string, the first half of the quotes were for
    // opening the string and the rest were for closing it
    let real_num_quotes = num_quotes / 2;
    if real_num_quotes % 2 == 0 {
      // Only an odd number of quotes can start/end a string
      return Err(TokenizeError::IncorrectOpeningQuotes { span: Span { start, end } });
    } else {
      return Ok(Token {
        token_type: TokenType::QuotedString { delim_len: real_num_quotes * quote_len },
        span: Span { start, end },
      });
    }
  }

  let try_parse_end_quotes = |chars: &mut Peekable<CharIndices>, end: &mut usize| {
    // When this function is called, we've already consumed one quote character
    let mut num_end_quotes = 1;
    loop {
      match chars.peek() {
        Some((_, c)) if *c == quote_char => {
          chars.next();
          *end += quote_len;
          num_end_quotes += 1;
        }
        _ => break,
      }
    }

    match num_end_quotes.cmp(&num_quotes) {
      Ordering::Less => Ok(None),
      Ordering::Equal => Ok(Some(Token {
        token_type: TokenType::QuotedString { delim_len: num_quotes * quote_len },
        span: Span { start, end: *end },
      })),
      Ordering::Greater => Err(TokenizeError::MismatchedEndingQuotes {
        open_span: Span { start, end: start + num_quotes * quote_len },
        end_span: Span { start: *end - num_end_quotes * quote_len, end: *end },
      }),
    }
  };

  while let Some((_, c)) = chars.next() {
    end += c.len_utf8();
    if c == quote_char {
      if let Some(tok) = try_parse_end_quotes(chars, &mut end)? {
        return Ok(tok);
      }
    } else if !is_raw && c == '\\' {
      let Some((_, next)) = chars.next() else {
        return Err(TokenizeError::NoEscapedCharacter { span: Span { start: end - 1, end } });
      };
      end += next.len_utf8();
    }
  }

  Err(TokenizeError::NoEndingQuote {
    open_span: Span { start, end: start + num_quotes * quote_len },
  })
}

mod test {
  // #[case]
  // fn tokenize_happy(#[case] text: &str, #[case] expected: Vec<Token>) {

  // }
}
