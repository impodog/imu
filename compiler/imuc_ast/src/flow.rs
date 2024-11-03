use crate::expr::{Body, Expr};

/// Syntax tree of flow controls
pub enum Flow {
    If(If),
    While(While),
}

/// An "if" statement stored in [`Flow`]
pub struct If {
    pub cond: Box<Expr>,
    pub body: Body,
}

/// A "while" statement stored in [`Flow`]
pub struct While {
    pub cond: Box<Expr>,
    pub body: Body,
}
