use crate::prelude::*;
use nonempty::NonEmpty;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub struct Body {
    name: String,
    list: Vec<cmd::Cmd>,
    stack: cmd::Ptr,
    locals: NonEmpty<super::Locals>,
}

impl Deref for Body {
    type Target = Vec<cmd::Cmd>;
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl DerefMut for Body {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.list
    }
}

impl Body {
    /// Creates an empty function body
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// Pushes bytes into the stack, returning the pointer to the top before pushing
    pub fn push_stack(&mut self, bytes: cmd::Bytes) -> cmd::Ptr {
        let result = self.stack;
        self.stack += bytes;
        result
    }

    /// Gets the pointer of stack top
    pub fn stack(&self) -> cmd::Ptr {
        self.stack
    }

    /// Creates a new group of locals at the back of the stack
    pub fn push_locals(&mut self) -> &mut super::Locals {
        self.locals.push(Default::default());
        self.locals.last_mut()
    }

    /// Deletes the last group of locals of the stack
    pub fn pop_locals(&mut self) -> Option<super::Locals> {
        self.locals.pop()
    }

    /// Gets the reference to the current locals
    pub fn locals(&self) -> &super::Locals {
        self.locals.last()
    }

    /// Gets the reference to the current locals
    pub fn locals_mut(&mut self) -> &mut super::Locals {
        self.locals.last_mut()
    }

    /// Gets an iterator over locals in top-first order
    pub fn locals_iter_rev(&self) -> impl Iterator<Item = &super::Locals> {
        self.locals.iter().rev()
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

    pub fn plus_name(&self, name: &str) -> StrRef {
        format!("{}.{}", self.name, name).into()
    }
}
