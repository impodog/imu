use crate::prelude::*;
use imuc_error::errors::ctx::SendError;
use imuc_ir::cmd::{GlobalPtr, Ptr};
use imuc_ir::sym::Ty;
use nonempty::NonEmpty;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

/// All context info used when converting AST to IR
///
/// An additional type map is present, containing all types with mangled names.
/// Searching directly is impossible, thus a vector of bodies should be used
pub struct Ctx {
    pub ty: super::Types,
    pub fun: super::Funs,
    pub globs: super::GlobsHandle,
    pub error_queue: Arc<RwLock<VecDeque<Error>>>,
    body: NonEmpty<super::Body>,
}

impl Ctx {
    /// Creates a new empty context of IR conversion
    pub fn new(base_name: String) -> Self {
        let glob = super::GlobsHandle::default();
        Self {
            ty: Default::default(),
            fun: Default::default(),
            globs: glob.clone(),
            error_queue: Default::default(),
            body: NonEmpty::new(super::Body::new(glob, base_name, None)),
        }
    }

    /// Gets the reference to the current function body
    pub fn body(&self) -> &super::Body {
        self.body.last()
    }

    /// Gets the mutable reference to the current function body
    pub fn body_mut(&mut self) -> &mut super::Body {
        self.body.last_mut()
    }

    /// Gets the reference to the bottom function body
    pub fn bottom(&self) -> &super::Body {
        self.body.first()
    }

    /// Gets the mutable reference to the bottom function body
    pub fn bottom_mut(&mut self) -> &mut super::Body {
        self.body.first_mut()
    }

    /// Pushes a new body of function when entering inner functions
    pub fn push_body(&mut self, name: &str, self_ty: Option<Ty>) -> &mut super::Body {
        let base_name = self.body.last().name();
        let name = format!("{base_name}.{name}");

        self.body
            .push(super::Body::new(self.globs.clone(), name, self_ty));
        self.body.last_mut()
    }

    /// Gets the last body of functions being pushed, dropping all locals
    pub fn pop_body(&mut self) -> Option<super::Body> {
        if let Some(mut body) = self.body.pop() {
            while body.pop_locals().is_some() {}
            Some(body)
        } else {
            None
        }
    }

    /// Gets reference to the nearest type of given name
    pub fn get_type(&self, name: &str) -> Option<&sym::Ty> {
        let body = self.body.last();
        for locals in body.locals_iter_rev() {
            if let Some(ty) = locals.ty.get(name) {
                return Some(ty);
            }
        }
        self.ty.get(name)
    }

    /// Gets a new [`GlobalPtr`] to a specified position in the current stack
    pub fn get_global_ptr(&self, ptr: Ptr) -> GlobalPtr {
        GlobalPtr::new(self.body.len() as u32, ptr)
    }

    /// Merges the functions from an iterator, same as calling on [`Self::glob`],
    /// but with the body parameter given, preventing reference errors
    pub fn merge_fun<'a, I>(&mut self, funs: I)
    where
        I: IntoIterator<Item = (&'a StrRef, &'a sym::FunSig)>,
    {
        self.globs
            .write()
            .unwrap()
            .merge_fun(self.body.last_mut(), funs);
    }

    /// Accesses [`Self::error_queue`] and pushes back an error
    pub fn push_error(&self, error: impl Into<Error>) {
        self.error_queue.write().unwrap().push_back(error.into());
    }

    /// Returns a function applicable to [`Result::map_err`] that grabs the stored error
    /// and replaces it with a [`SendError`]
    pub fn push_error_fn<E>(&self) -> impl Fn(E) -> SendError + 'static
    where
        E: Into<Error>,
    {
        let error_queue = self.error_queue.clone();
        move |error: E| {
            error_queue.write().unwrap().push_back(error.into());
            SendError::default()
        }
    }
}
