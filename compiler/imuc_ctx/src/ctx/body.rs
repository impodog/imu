use crate::prelude::*;
use imuc_ir::sym::Ty;
use nonempty::NonEmpty;
use std::ops::{Deref, DerefMut};

pub struct Body {
    pub globs: super::GlobsHandle,
    name: String,
    self_ty: Vec<Ty>,
    list: Vec<cmd::Cmd>,
    stack: cmd::Ptr,
    stack_record: Vec<cmd::Ptr>,
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
            self_ty: if let Some(self_ty) = self_ty {
                vec![self_ty]
            } else {
                Vec::new()
            },
            list: Default::default(),
            stack: Default::default(),
            stack_record: Default::default(),
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

    /// Memorize the current stack pointer, to be reverted later
    pub fn push_stack_record(&mut self) {
        self.stack_record.push(self.stack());
    }

    /// Pops the most recent stack pointer and reverts to it, if any. Returns true if successful
    pub fn pop_stack_record(&mut self) -> bool {
        if let Some(stack) = self.stack_record.pop() {
            self.stack = stack;
            true
        } else {
            false
        }
    }

    /// Reverts to the most recent stack pointer, if any. Returns true if successful
    pub fn revert_stack_record(&mut self) -> bool {
        if let Some(stack) = self.stack_record.last() {
            self.stack = *stack;
            true
        } else {
            false
        }
    }

    pub fn stack_record(&self) -> Option<cmd::Ptr> {
        self.stack_record.last().copied()
    }

    /// Creates a new group of locals at the back of the stack
    pub fn push_locals(&mut self) -> &mut super::Locals {
        self.locals.push(Default::default());
        self.locals.last_mut()
    }

    /// Deletes the last group of locals of the stack, also drops all assigned locals.
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
        self.self_ty.last()
    }

    /// Pushes a new layer of self_ty, making it the current available type
    pub fn push_self_ty(&mut self, ty: Ty) {
        self.self_ty.push(ty);
    }

    /// Removes the last layer of self_ty, if any
    pub fn pop_self_ty(&mut self) -> Option<Ty> {
        self.self_ty.pop()
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

    /// Adds a list of commands into current list. This is not commonly used
    pub fn extend_cmd(&mut self, cmd: impl IntoIterator<Item = cmd::Cmd>) {
        self.list.extend(cmd);
    }
}
