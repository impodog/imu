use crate::prelude::*;
use nonempty::NonEmpty;

/// All context info used when converting AST to IR
#[derive(Default)]
pub struct Ctx {
    pub ty: super::Types,
    body: Vec<Vec<cmd::Cmd>>,
    locals: NonEmpty<super::Locals>,
    /// The pointer to the top of current stack
    stack: cmd::Ptr,
}

impl Ctx {
    /// Creates a new empty context of IR conversion
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets reference to the nearest type of given name
    pub fn get_type(&self, name: &str) -> Option<&sym::Ty> {
        for locals in self.locals.iter().rev() {
            if let Some(ty) = locals.ty.get(name) {
                return Some(ty);
            }
        }
        self.ty.get(name)
    }

    /// Gets reference to the nearest value of given name
    pub fn get_value(&self, name: &str) -> Option<&super::Value> {
        for locals in self.locals.iter().rev() {
            if let Some(value) = locals.value.get(name) {
                return Some(value);
            }
        }
        None
    }

    /// Creates a new group of locals at the back of the stack
    pub fn push_locals(&mut self) -> &mut super::Locals {
        self.locals.push(Default::default());
        self.locals.last_mut()
    }

    /// Deletes the last group of locals of the stack
    pub fn pop_locals(&mut self) -> Option<super::Locals> {
        self.locals
            .pop()
            .inspect(|locals| self.stack -= locals.stack)
    }

    /// Gets the reference to the current locals
    pub fn locals(&self) -> &super::Locals {
        self.locals.last()
    }

    /// Gets the reference to the current locals
    pub fn locals_mut(&mut self) -> &mut super::Locals {
        self.locals.last_mut()
    }

    /// Pushes bytes into the stack pointer, returning the stack pointer before pushing
    pub fn push_stack(&mut self, bytes: cmd::Bytes) -> cmd::Ptr {
        let result = self.stack;
        self.stack += bytes;
        self.locals.last_mut().stack += bytes;
        result
    }

    /// Gets the reference to the current function body
    pub fn body_mut(&mut self) -> Option<&mut Vec<cmd::Cmd>> {
        self.body.last_mut()
    }

    /// Gets the reference to the current function body, or generates an error
    pub fn body_mut_or(&mut self) -> Result<&mut Vec<cmd::Cmd>> {
        self.body_mut()
            .ok_or(errors::ConvError::FunctionRequired.into())
    }

    /// Pushes a new body of function when entering inner functions
    pub fn push_body(&mut self) {
        self.body.push(Default::default())
    }

    /// Gets the last body of functions being pushed
    pub fn pop_body(&mut self) -> Option<Vec<cmd::Cmd>> {
        self.body.pop()
    }
}
