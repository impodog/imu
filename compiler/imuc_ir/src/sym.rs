use crate::prelude::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
/// A compiler data group from symbols that appear during compilation
pub struct SymTable {
    map: HashMap<StrRef, Sym>,
}

impl Deref for SymTable {
    type Target = HashMap<StrRef, Sym>;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for SymTable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// A single symbol corresponding to a name
#[derive(Clone)]
pub enum Sym {
    Bind(crate::ty::Ty),
    Fun(crate::fun::Fun),
    Ty(crate::ty::Ty),
    Label(usize),
}
