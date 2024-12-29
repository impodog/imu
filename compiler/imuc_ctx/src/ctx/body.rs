use crate::prelude::*;
use std::ops::{Deref, DerefMut};

#[derive(Default, Clone)]
pub struct Body {
    list: Vec<cmd::Cmd>,
    stack: cmd::Ptr,
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
    pub fn new() -> Self {
        Self::default()
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
}
