use std::collections::HashMap;

/// The start and offset of a [Tree]
pub struct Span {
    pub start: usize,
    pub end: usize
}

pub enum TypeTag {
    Int,
    Str,
    Custom(String)
}

pub enum Tree {
    Str(String),
    Num(String),
    Bool(bool),
    List(Vec<TreeInfo>),
    Map(HashMap<TreeInfo, TreeInfo>)
}

pub struct TreeInfo {
    pub tree: Tree,
    pub tpe: TypeTag,
    pub span: Span
}
