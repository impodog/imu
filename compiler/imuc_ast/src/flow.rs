use crate::expr::{Body, Expr};

/// Syntax tree of flow controls
pub enum Flow {
    If(If),
    Loop(Loop),
}

/// An "if" statement stored in [`Flow`]
pub struct If {
    pub cond: Box<Expr>,
    pub body: Body,
}

/// A "loop" statement stored in [`Flow`]
pub struct Loop {
    pub body: Body,
}
