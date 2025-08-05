use crate::prelude::*;
use imuc_ir::sym::Ty;
use nonempty::NonEmpty;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;

pub struct Body {
    pub globs: super::GlobsHandle,
    name: String,
    self_ty: Vec<Ty>,
    list: Vec<cmd::Cmd>,
    stack: cmd::Ptr,
    /// Pointers to the past stack pointers when entering a body
    stack_record: Vec<cmd::Ptr>,
    /// Pointers to the loop quit handle command
    loop_record: Vec<LoopRecord>,
    locals: NonEmpty<super::Locals>,
    pub(super) imports: super::Imports,
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
    pub(crate) fn new(
        globs: super::GlobsHandle,
        name: String,
        self_ty: Option<Ty>,
        imports: super::Imports,
    ) -> Self {
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
            loop_record: Default::default(),
            locals: Default::default(),
            imports,
        }
    }

    /// Pushes bytes into the stack, returning the pointer to the top before pushing
    pub fn push_stack(&mut self, bytes: cmd::Bytes) -> cmd::Ptr {
        // FIXME: This is temporary, and wastes much memory. Fix?
        self.align_stack();
        let result = self.stack;
        self.stack += bytes;
        result
    }

    /// Gets the pointer to stack top
    pub fn stack(&self) -> cmd::Ptr {
        self.stack
    }

    /// Aligns the stack pointer to `crate::config::MEMORY_LAYOUT.ptr_align` and returns the
    /// aligned one, wasting some memory if necessary
    pub fn align_stack(&mut self) -> cmd::Ptr {
        let rem = self.stack.num() % crate::config::MEMORY_LAYOUT.ptr_align;
        if rem != 0 {
            let add = crate::config::MEMORY_LAYOUT.ptr_align - rem;
            self.stack += cmd::Bytes::new(add);
        }
        self.stack
    }

    /// Memorize the current stack pointer, to be reverted later
    pub fn push_stack_record(&mut self) {
        // Aligns the stack for return value
        self.align_stack();
        // debug!("Push stack record: {:?}", self.stack());
        self.stack_record.push(self.stack());
    }

    /// Reverts to the given stack pointer, asserting it is lower than or equal to current stack
    pub fn revert_stack_record_to(&mut self, stack: cmd::Ptr) {
        // debug!("Revert stack record: {:?} to {:?}", self.stack(), stack);
        debug_assert!(self.stack >= stack);
        self.force_stack_to(stack);
    }

    /// Forces to revert stack to the given pointer, given that the current stack should be larger
    /// than the stack pointer. This is only used after loop statements since there are special jump
    /// commands by mit
    pub fn force_stack_to(&mut self, stack: cmd::Ptr) {
        // debug!("Force stack to {stack:?}");
        self.push(cmd::Cmd::Shrink(stack));
        self.stack = stack;
    }

    /// Pops the most recent stack pointer and reverts to it plus given bytes, if any. Returns true if successful
    pub fn pop_stack_record(&mut self, plus: cmd::Bytes) -> bool {
        if let Some(stack) = self.stack_record.pop() {
            self.revert_stack_record_to(stack + plus);
            true
        } else {
            false
        }
    }

    /// Reverts to the most recent stack pointer plus given bytes, if any. Returns true if successful
    pub fn revert_stack_record(&mut self, plus: cmd::Bytes) -> bool {
        if let Some(stack) = self.stack_record.last() {
            self.revert_stack_record_to(*stack + plus);
            true
        } else {
            false
        }
    }

    /// Gets the last stack record pointer stored
    pub fn stack_record(&self) -> Option<cmd::Ptr> {
        self.stack_record.last().copied()
    }

    /// Memorize the *next* (yet to push) cmd pointer in the loop records, so that "mit" expressions can be
    /// evaluated and jumped properly
    pub fn push_loop_record(&mut self) {
        /// Aligns the stack for return value
        self.align_stack();
        let ptr = cmd::Ptr::new(self.len());
        self.loop_record.push(LoopRecord {
            ptr,
            stack: self.stack(),
            ty: Default::default(),
        });
    }

    /// Pops the most recent loop pointer, if any.
    /// Note this does not revert the stack, and you have to do it manually
    pub fn pop_loop_record(&mut self) -> Option<LoopRecord> {
        self.loop_record.pop()
    }

    /// Gets the nth most recent loop record pointer stored, but you have to revert manually
    pub fn loop_record(&self, index: usize) -> Option<&LoopRecord> {
        if let Some(index) = self.loop_record.len().checked_sub(index + 1) {
            let loop_record = self
                .loop_record
                .get(index)
                .expect("value should exist since the index is lower than length");
            Some(loop_record)
        } else {
            None
        }
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

    /// Gets the name from the alias of an imported item, from most recent imports to outer bodies
    pub fn get_import(&self, name: &str) -> Option<StrRef> {
        // NOTE: Because Body can only be created by Ctx, it is guaranteed to have safe order of
        // imports stack pointers, so this function is safe
        unsafe { self.imports.query(name) }
    }

    /// Inserts a import value into the map, if not already.
    pub fn insert_import(&mut self, alias: StrRef, value: StrRef) -> bool {
        self.imports.insert(alias, value)
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
    pub fn take_cmd(&mut self) -> Vec<cmd::Cmd> {
        std::mem::take(&mut self.list)
    }

    /// Adds a list of commands into current list. This is not commonly used
    pub fn extend_cmd(&mut self, cmd: impl IntoIterator<Item = cmd::Cmd>) {
        self.list.extend(cmd);
    }
}

/// Records the loop related pointers responsible for handling "mit" expressions.
/// `ptr` is the index to the command that jumps out of the loop
/// `stack` is the stack location before the loop starts
/// `ty` is the return type of the loop, and will be initialized when a "mit" is first met
pub struct LoopRecord {
    pub ptr: cmd::Ptr,
    pub stack: cmd::Ptr,
    pub ty: OnceLock<Ty>,
}

impl LoopRecord {
    /// Check if the return type of the loop is the same as given or undetermined,
    /// If undetermined, the type is set to the given type
    pub fn check_ty(&self, ty: &Ty) -> bool {
        let other = self.ty.get_or_init(|| ty.clone());
        other.test_eq(ty)
    }
}
