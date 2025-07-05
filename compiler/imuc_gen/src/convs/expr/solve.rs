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

impl ExprSolver {
    /// Creates a new solver that only inherits fields that are necessary and safe to inherit
    pub fn inherit(solver: &ExprSolver) -> ExprSolver {
        Self {
            hint: solver.hint.clone(),
            ..Default::default()
        }
    }

    /// Modifies the hint field and return self
    pub fn with_hint(mut self, hint: Option<Ty>) -> Self {
        self.hint = hint;
        self
    }

    /// Modifies the hint field if not already given and return self
    pub fn with_hint_or_else(mut self, hint: Option<Ty>) -> Self {
        if self.hint.is_none() {
            self.hint = hint;
        }
        self
    }
}
