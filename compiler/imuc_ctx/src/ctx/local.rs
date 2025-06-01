use crate::prelude::*;
use std::collections::BTreeMap;

/// Information required to compile a value object
#[derive(Debug, Clone)]
pub struct Value {
    pub ty: sym::Ty,
    pub ptr: cmd::Ptr,
}

impl Default for Value {
    fn default() -> Self {
        Value {
            ty: sym::Ty::unit(),
            ptr: cmd::Ptr::default(),
        }
    }
}

/// Local names references in an AST body
///
/// Another map of types in present, allowing searching types without mangled names
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
