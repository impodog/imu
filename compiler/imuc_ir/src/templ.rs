use crate::prelude::*;
use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};

type Map = BTreeMap<StrRef, Option<crate::ty::Ty>>;

/// Template variables for IR items, uses type maps internally
pub struct Templ {
    map: Map,
}

impl Deref for Templ {
    type Target = Map;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for Templ {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}
