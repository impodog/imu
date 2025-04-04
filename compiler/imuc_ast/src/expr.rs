use imuc_derive::Spanned;
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
    Cus(Cus),
}

pub enum Value {
    Unused,
    Name(imuc_lexer::StrRef),
    Res(imuc_lexer::token::ResVal),
}

/// An expression with a unary operator
#[derive(Spanned)]
pub struct UnExpr {
    pub op: UnOp,
    pub val: Box<Expr>,
    pub span: crate::Span,
}

/// An expression with a binary operator
#[derive(Spanned)]
pub struct BinExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: crate::Span,
}

/// A group of expressions and/or bindings wrapped in braces as a body
#[derive(Spanned)]
pub struct Body {
    pub bind: Vec<crate::bind::Bind>,
    pub body: Vec<Expr>,
    pub unit: bool,
    pub span: crate::Span,
}

#[derive(Spanned)]
pub struct Tuple {
    pub elem: Vec<Expr>,
    pub span: crate::Span,
}

#[derive(Spanned)]
pub struct Cus {
    pub ty: crate::pat::Type,
    pub elem: BTreeMap<imuc_lexer::StrRef, Expr>,
    pub span: crate::Span,
}
