use crate::prelude::*;
use imuc_ir::sym::Fun;
use std::collections::BTreeMap;
use std::ops::Deref;

type FunMap = BTreeMap<StrRef, Fun>;

/// Stores a map of in-module functions that can be used to build a `Module`][`imuc_ir::module::Module`
#[derive(Default)]
pub struct Funs {
    map: FunMap,
}

impl Deref for Funs {
    type Target = FunMap;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Funs {
    /// Creates a new, empty fun map
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a function into the map
    pub fn insert(&mut self, name: StrRef, fun: Fun) {
        self.map.insert(name, fun);
    }

    /// Extracts the map stored, consuming the struct
    pub fn into_map(self) -> FunMap {
        self.map
    }
}
