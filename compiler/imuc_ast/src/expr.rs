use imuc_lexer::token::{BinOp, UnOp};
use std::collections::BTreeMap;

/// Syntax tree of different types of expressions
pub enum Expr {
    Prim(crate::prim::Prim),
    Value(Value),
    UnExpr(UnExpr),
    BinExpr(BinExpr),
    Body(Body),
    Flow(crate::flow::Flow),
    Tuple(Tuple),
    Struct(Struct),
}

pub enum Value {
    Unused,
    Name(crate::StrRef),
    Res(imuc_lexer::token::ResVal),
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
    pub bind: Vec<crate::bind::Bind>,
    pub body: Vec<Expr>,
    pub unit: bool,
}

pub struct Tuple {
    pub elem: Vec<Expr>,
}

pub struct Struct {
    pub ty: crate::pat::Type,
    pub elem: BTreeMap<crate::StrRef, Expr>,
}
