/// The start and offset of a [Tree]
#[derive(Clone, Copy)]
pub struct Span {
  pub start: usize,
  pub end: usize,
}

pub enum Tree {
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
    val: Vec<Tree>,
    span: Span,
  },
  Map {
    val: Vec<(Tree, Tree)>,
    span: Span,
  },
}

pub struct Num {
  pub integer_part: String,
  pub decimal_part: String,
  pub exponent: String,
}

pub enum QuotedStringType {
  Unindent,
  SingleLine,
}
