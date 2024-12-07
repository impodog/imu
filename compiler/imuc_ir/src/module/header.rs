use crate::prelude::*;
use crate::sym::*;
use std::collections::BTreeMap;

/// Header for a module, defining interfaces of types and functions
///
/// This is used when referencing to a external module. For compiling new modules, [`Module`](`crate::module::Module`) is used
pub struct Header {
    pub ty: BTreeMap<StrRef, Ty>,
    pub fun: BTreeMap<StrRef, FunSig>,
}
