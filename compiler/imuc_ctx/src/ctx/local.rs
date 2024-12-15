use crate::prelude::*;
use std::collections::BTreeMap;

/// Information required to compile a value object
pub struct Value {
    pub ty: sym::Ty,
    pub ptr: cmd::Ptr,
}

/// Local names references in an AST body
#[derive(Default)]
pub struct Locals {
    pub ty: super::Types,
    pub value: BTreeMap<StrRef, Value>,
}

impl Value {
    pub fn new(ty: sym::Ty, ptr: cmd::Ptr) -> Self {
        Self { ty, ptr }
    }
}

impl Locals {
    pub fn new() -> Self {
        Self::default()
    }
}
