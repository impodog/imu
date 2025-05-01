use imuc_derive::Spanned;
use imuc_lexer::token::{BinOp, UnOp};
use std::collections::BTreeMap;

/// Syntax tree of different types of expressions
pub enum Expr {
    Prim(crate::prim::Prim),
    Value(Value),
    Call(Call),
    UnExpr(UnExpr),
    BinExpr(BinExpr),
    Body(Body),
    Flow(crate::flow::Flow),
    Tuple(Tuple),
    Cus(Cus),
}

impl Expr {
    /// Gets the span of the element of this expression, if any.
    /// Some expression variants do not contain a span, because they do not cause any errors by
    /// themselves, and will never become valueless
    pub fn span(&self) -> Option<imuc_lexer::Span> {
        let span = match self {
            Self::Prim(_prim) => return None,
            Self::Value(value) => value.span(),
            Self::Call(call) => call.span(),
            Self::UnExpr(un_expr) => un_expr.span(),
            Self::BinExpr(bin_expr) => bin_expr.span(),
            Self::Body(body) => body.span(),
            Self::Flow(_flow) => return None,
            Self::Tuple(tuple) => tuple.span(),
            Self::Cus(cus) => cus.span(),
        };
        Some(span)
    }

    /// Gets the span of the element of this expression.
    /// Some expression variants do not contain a span, because they do not cause any errors by
    /// themselves, and will never become valueless.
    /// If you call this, you must make sure that the expression *can* cause errors, and therefore
    /// contains a span
    pub fn unwrap_span(&self) -> imuc_lexer::Span {
        self.span()
            .expect("the span should exist as user called this on never valueless expressions")
    }
}

/// An expression of name token or reserved value
#[derive(Spanned)]
pub struct Value {
    pub value: ValueInner,
    pub span: imuc_lexer::Span,
}

#[derive(Spanned)]
pub struct Call {
    pub func: Box<Expr>,
    pub args: Box<Expr>,
    pub span: imuc_lexer::Span,
}

pub enum ValueInner {
    Unused,
    Name(imuc_lexer::StrRef),
    Res(imuc_lexer::token::ResVal),
}

/// An expression with a unary operator
#[derive(Spanned)]
pub struct UnExpr {
    pub op: UnOp,
    pub val: Box<Expr>,
    pub span: imuc_lexer::Span,
}

/// An expression with a binary operator
#[derive(Spanned)]
pub struct BinExpr {
    pub op: BinOp,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
    pub span: imuc_lexer::Span,
}

/// A group of expressions and/or bindings wrapped in braces as a body
#[derive(Spanned)]
pub struct Body {
    pub bind: Vec<crate::bind::Bind>,
    pub body: Vec<Expr>,
    pub unit: bool,
    pub span: imuc_lexer::Span,
}

#[derive(Spanned)]
pub struct Tuple {
    pub elem: Vec<Expr>,
    pub span: imuc_lexer::Span,
}

#[derive(Spanned)]
pub struct Cus {
    pub ty: crate::pat::Type,
    pub elem: BTreeMap<imuc_lexer::StrRef, Expr>,
    pub span: imuc_lexer::Span,
}
