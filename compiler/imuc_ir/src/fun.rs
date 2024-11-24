use crate::prelude::*;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

/// A function handle pointing at a certain loaded function
///
/// This can be used as [`Option<Fun>`] without memory overhead
#[derive(Clone)]
pub struct Fun(Arc<FunInner>);
impl Deref for Fun {
    type Target = FunInner;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

/// An IR map representing a portion of the functions in a file,
/// mapping from str to [`Fun`]
pub struct FunMap {
    file: File,
    pub map: BTreeMap<StrRef, Fun>,
}

impl FunMap {
    /// Creates a new empty map of functions
    pub fn new(file: File) -> Self {
        Self {
            file,
            map: Default::default(),
        }
    }

    pub fn file(&self) -> &File {
        &self.file
    }
}

/// IR representation of any function
pub struct FunInner {
    pub name: StrRef,
    pub templ: crate::templ::Templ,
    pub body: crate::cmd::CmdBody,
}
