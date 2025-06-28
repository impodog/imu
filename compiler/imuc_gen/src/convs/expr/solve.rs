use crate::prelude::*;
use std::sync::{Arc, OnceLock};

/// A set of utility to help guide the behavior of expression solver
#[derive(Default)]
pub struct ExprSolver {
    /// Hint type of the expression
    pub hint: Option<Ty>,
    /// Stores the self value, if required
    /// When setting to Some, you *must* make sure that the underlying expression IS a dot operator
    pub self_value: Option<Arc<OnceLock<Value>>>,
}
