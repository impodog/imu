use crate::prelude::*;
use imuc_error::errors::ctx::SendError;
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
    pub error_queue: Arc<RwLock<VecDeque<errors::ctx::ConvError>>>,
    pub import_pool: super::ImportPoolHandle,
    body: NonEmpty<super::Body>,
}

impl Drop for Ctx {
    fn drop(&mut self) {
        // NOTE: Ensures safe behavior, dropping imports from the top of the stack first
        while self.body.pop().is_some() {}
    }
}

impl Ctx {
    /// Creates a new empty context of IR conversion
    pub fn new(base_name: String) -> Self {
        let glob = super::GlobsHandle::default();
        let imports = super::Imports::default();
        Self {
            ty: Default::default(),
            fun: Default::default(),
            globs: glob.clone(),
            error_queue: Default::default(),
            import_pool: Default::default(),
            body: NonEmpty::new(super::Body::new(glob, base_name, None, imports)),
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

    /// Pushes a new body of function when entering inner functions or modules
    pub fn push_body(
        &mut self,
        name: &str,
        // Defines whether to directly use the given name and not mangle it again
        direct_name: bool,
        self_ty: Option<Ty>,
        is_module: bool,
    ) -> &mut super::Body {
        let base_name = self.body.last().name();
        let body_name = if direct_name {
            name.to_owned()
        } else {
            super::mangle::mangle_inside_body(base_name, name)
        };
        let imports = if is_module {
            super::Imports::default()
        } else {
            // NOTE: This ensures safe behavior where imports are only removed by stack order
            super::Imports::new_under(&mut self.body.last_mut().imports)
        };

        self.body.push(super::Body::new(
            self.globs.clone(),
            body_name,
            self_ty,
            imports,
        ));
        self.body.last_mut()
    }

    /// Gets the last body of functions being pushed, popping all locals bound
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
    pub fn push_error(&self, error: errors::ctx::ConvError) {
        self.error_queue.write().unwrap().push_back(error);
    }

    /// Returns a function applicable to [`Result::map_err`] that grabs the stored error
    /// and replaces it with a [`SendError`]
    pub fn push_error_fn(&self) -> impl Fn(errors::ctx::ConvError) -> SendError + 'static {
        let error_queue = self.error_queue.clone();
        move |error| {
            error_queue.write().unwrap().push_back(error);
            SendError::default()
        }
    }

    /// Creates the entry point function for the module with given name.
    /// This takes the bottom body and pushes the function into current map
    pub fn make_entry_fun(&mut self) {
        let name = StrRef::from(crate::ctx::mangle::mangle_entry(self.body.first().name()));
        let cmd = self.bottom_mut().take_cmd();

        let param_ty = Ty::unit();
        let ret_ty = Ty::unit();

        let fun_ty_name = StrRef::from(super::mangle::mangle_fun_sig(name.as_str()));
        let fun_ty = self
            .ty
            .or_insert_with(fun_ty_name.clone(), || {
                Ty::new(sym::ty::TyInner::new(
                    fun_ty_name,
                    sym::ty::TyKind::Fun {
                        param: param_ty.clone().into(),
                        ret: ret_ty.clone().into(),
                    },
                ))
            })
            .clone();

        let fun_ptr_ty_name = StrRef::from(super::mangle::mangle_ptr(fun_ty.name.as_str()));
        let _fun_ptr_ty = self
            .ty
            .or_insert_with(fun_ptr_ty_name.clone(), move || {
                Ty::new(sym::ty::TyInner::new(
                    fun_ptr_ty_name,
                    sym::ty::TyKind::Ptr(fun_ty.into()),
                ))
            })
            .clone();
        self.fun.insert(
            name.clone(),
            sym::Fun {
                body: cmd::CmdBody::new(cmd),
                name,
                sig: sym::FunSig {
                    param: sym::ty::TyItem::Solid(param_ty),
                    ret: sym::ty::TyItem::Solid(ret_ty),
                },
            },
        );
    }
}
