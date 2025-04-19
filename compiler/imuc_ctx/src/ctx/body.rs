use crate::prelude::*;
use imuc_ir::sym::Ty;
use nonempty::NonEmpty;
use std::ops::{Deref, DerefMut};

pub struct Body {
    pub globs: super::GlobsHandle,
    name: String,
    self_ty: Option<Ty>,
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
    pub fn new(globs: super::GlobsHandle, name: String, self_ty: Option<Ty>) -> Self {
        Self {
            globs,
            name,
            self_ty,
            list: Default::default(),
            stack: Default::default(),
            locals: Default::default(),
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

    /// Deletes the last group of locals of the stack, also drops all assigned locals
    pub fn pop_locals(&mut self) -> Option<super::Locals> {
        if let Some(locals) = self.locals.pop() {
            for (_, value) in locals.value.iter() {
                self.drop_value(value).expect("The type should exist");
            }
            Some(locals)
        } else {
            None
        }
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

    pub fn with_globs<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self, &super::Globs) -> R,
    {
        let globs = self.globs.clone();
        let lock = globs.read().unwrap();
        f(self, &lock)
    }

    pub fn with_globs_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self, &mut super::Globs) -> R,
    {
        let globs = self.globs.clone();
        let mut lock = globs.write().unwrap();
        f(self, &mut lock)
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

    /// Gets the self type context, if any
    pub fn self_ty(&self) -> Option<&Ty> {
        self.self_ty.as_ref()
    }

    /// Mangle the name of elements for types
    pub fn mangle_name(&self, name: &str) -> StrRef {
        format!("{}.{}", self.name, name).into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Takes the list of cmd of the body when exporting as a function
    pub fn take_cmd(self) -> Vec<cmd::Cmd> {
        self.list
    }
}
