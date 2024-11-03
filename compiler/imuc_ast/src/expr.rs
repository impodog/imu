use imuc_lexer::token::{BinOp, UnOp};
use std::collections::BTreeMap;

/// Syntax tree of different types of expressions
pub enum Expr {
    Prim(crate::prim::Prim),
    UnExpr(UnExpr),
    BinExpr(BinExpr),
    Flow(crate::flow::Flow),
    Tuple(Tuple),
    Struct(Struct),
}

/// An expression with a unary operator
pub struct UnExpr {
    pub op: UnOp,
    pub val: Box<Expr>,
}

/// An expression with a binary operator
pub struct BinExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

/// A group of expressions and/or bindings wrapped in braces as a body
pub struct Body {
    pub body: Vec<BodyElem>,
    pub unit: bool,
}

pub enum BodyElem {
    Expr(Expr),
    Bind(crate::bind::Bind),
}

pub struct Tuple {
    pub elem: Vec<Expr>,
}

pub struct Struct {
    pub elem: BTreeMap<String, Expr>,
}
