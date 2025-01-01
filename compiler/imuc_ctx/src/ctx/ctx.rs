use crate::prelude::*;

/// All context info used when converting AST to IR
pub struct Ctx {
    pub ty: super::Types,
    base_name: String,
    body: Vec<super::Body>,
}

impl Ctx {
    /// Creates a new empty context of IR conversion
    pub fn new(base_name: String) -> Self {
        Self {
            ty: Default::default(),
            base_name,
            body: Default::default(),
        }
    }

    /// Gets the reference to the current function body
    pub fn body_mut(&mut self) -> Option<&mut super::Body> {
        self.body.last_mut()
    }

    /// Gets the reference to the current function body, or generates an error
    pub fn body_mut_or(&mut self) -> Result<&mut super::Body> {
        self.body_mut()
            .ok_or(errors::ConvError::FunctionRequired.into())
    }

    /// Pushes a new body of function when entering inner functions
    pub fn push_body(&mut self) -> &mut super::Body {
        self.body.push(Default::default());
        self.body.last_mut().unwrap()
    }

    /// Gets the last body of functions being pushed
    pub fn pop_body(&mut self) -> Option<super::Body> {
        self.body.pop()
    }

    /// Gets reference to the nearest type of given name
    pub fn get_type(&self, name: &str) -> Option<&sym::Ty> {
        if let Some(body) = self.body.last() {
            for locals in body.locals_iter_rev() {
                if let Some(ty) = locals.ty.get(name) {
                    return Some(ty);
                }
            }
        }
        self.ty.get(name)
    }
}
