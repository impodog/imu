use crate::expr::{Body, Expr};
use imuc_derive::Spanned;

/// Syntax tree of flow controls
pub enum Flow {
    IfElse(IfElse),
    Loop(Loop),
}

impl Flow {
    pub fn span(&self) -> imuc_lexer::Span {
        match self {
            Self::IfElse(ifelse_stmt) => ifelse_stmt.span(),
            Self::Loop(loop_stmt) => loop_stmt.span(),
        }
    }
}

/// An "if" statement stored in `Flow`
#[derive(Spanned)]
pub struct If {
    pub cond: Box<Expr>,
    pub body: Body,
    pub span: imuc_lexer::Span,
}

/// A series of "if-else if-else" represented in an array of ifs
#[derive(Spanned)]
pub struct IfElse {
    /// Conditions first handled
    pub ifs: nonempty::NonEmpty<If>,
    /// The final "else" statement, if any
    pub end: Option<Body>,
    pub span: imuc_lexer::Span,
}

/// A "loop" statement stored in `Flow`
#[derive(Spanned)]
pub struct Loop {
    pub body: Body,
    pub span: imuc_lexer::Span,
}
