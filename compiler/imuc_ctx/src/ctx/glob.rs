use super::{Body, Value};
use crate::prelude::*;
use cmd::{Bytes, Cmd, Ptr};
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use sym::{
    ty::{TyInner, TyKind},
    FunSig, Ty,
};

type GlobMap = HashMap<StrRef, Glob>;

/// A shared lock to the globals
pub type GlobsHandle = Arc<RwLock<Globs>>;

/// A global variable that can be accessed or created in the current module
///
/// If [`Self::external`] is set to true, this will not be exported,
/// otherwise the corresponding global will be written to the output file
///
/// [`Self::ty`] is the bare type of the global. However most of the time a ptr wrapping this ty is required
///
/// You can omit the type checking when acquiring a global, because the name implies the type
pub struct Glob {
    ptr: Ptr,
    ty: Ty,
    external: bool,
}

impl Glob {
    pub fn new(ptr: Ptr, ty: Ty, external: bool) -> Self {
        Self { ptr, ty, external }
    }

    pub fn ptr(&self) -> Ptr {
        self.ptr
    }

    pub fn ty(&self) -> &Ty {
        &self.ty
    }

    pub fn external(&self) -> bool {
        self.external
    }
}

/// A map used in [`Ctx`](`super::Ctx`) for compiling global variables with a static, mangled name
///
/// This also stores a ptr to the top of globals stack
#[derive(Default)]
pub struct Globs {
    fun: GlobMap,
    stack: Ptr,
}

impl Deref for Globs {
    type Target = GlobMap;
    fn deref(&self) -> &Self::Target {
        &self.fun
    }
}

impl Globs {
    /// Inserts a ptr to a global with a key into the global map
    ///
    /// TODO: This is not made global, because general global exporting is not supported at present
    fn insert(&mut self, name: StrRef, glob: Glob) {
        self.fun.insert(name, glob);
    }

    fn load_fun(&mut self, body: &mut Body, name: StrRef, ty: Ty) {
        let ptr = self.stack;
        body.push(Cmd::Link(name.clone()));
        self.stack += Bytes::ptr();
        self.insert(name, Glob::new(ptr, ty, true));
    }

    /// Merges an iterator of functions into globals, assigning each with a ptr, if not already
    ///
    /// Normally, the argument of this function is read from header files
    pub fn merge_fun<'a, I>(&mut self, body: &mut Body, funs: I)
    where
        I: IntoIterator<Item = (&'a StrRef, &'a FunSig)>,
    {
        for (name, sig) in funs.into_iter() {
            // Only unassigned names are used
            if !self.contains_key(name) {
                let ty = Ty::new(TyInner::new(
                    StrRef::from(crate::ctx::mangle::mangle_fun_sig(name)),
                    TyKind::Fun {
                        param: sig.param.clone().into(),
                        ret: sig.ret.clone().into(),
                    },
                ));
                self.load_fun(body, name.clone(), ty);
            }
        }
    }
}
