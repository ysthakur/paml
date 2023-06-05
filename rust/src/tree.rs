use std::collections::HashMap;

pub struct Tree {
  pub kind: TreeKind,
  pub span: Span
}

pub enum TreeKind {
  Scalar(Scalar),
  List(Vec<Tree>),
  Map(HashMap<Tree, Tree>)
}

pub enum Scalar {
  Str(String),
  Num(String),
  Bool(bool)
}

pub struct Span(usize, usize);

