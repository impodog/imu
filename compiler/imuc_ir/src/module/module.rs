use crate::prelude::*;
use std::collections::BTreeMap;

/// A complete module with types, functions and the defintions
pub struct Module {
    // header's function signatures are reused to in the new field, thus becoming useless
    header: crate::module::Header,
    pub fun: BTreeMap<StrRef, crate::sym::Fun>,
}
