use crate::prelude::*;
use std::collections::BTreeSet;

/// A function containing its name and commands
pub struct Fun {
    pub name: StrRef,
    pub sig: FunSig,
    pub body: crate::cmd::CmdBody,
}

/// The signature of a function, containing the parameter and return type, and templates
pub struct FunSig {
    pub templ: BTreeSet<super::TemplId>,
    pub param: super::Ty,
    pub ret: super::Ty,
}
