use crate::prelude::*;
use imuc_error::*;
use std::collections::BTreeMap;
use std::ops::Deref;

pub struct TemplItem {
    pub id: crate::consts::TemplId,
    pub ty: Option<crate::ty::Ty>,
}
type Map = BTreeMap<StrRef, TemplItem>;

/// Template variables for IR items, uses type maps internally
pub struct Templ {
    id: u8,
    map: Map,
}

impl Default for Templ {
    fn default() -> Self {
        Self {
            id: 1,
            map: Default::default(),
        }
    }
}

impl Templ {
    /// Adds a new template, returning its id
    pub fn insert(
        &mut self,
        name: StrRef,
        ty: Option<crate::ty::Ty>,
    ) -> Result<crate::consts::TemplId> {
        let id = self.id.try_into().expect("id shuold be greater than 0");
        self.id = self
            .id
            .checked_add(1)
            .ok_or_else(|| errors::MemoryError::OverflowError(u8::MAX as usize))?;
        self.map.insert(name, TemplItem { id, ty });
        Ok(id)
    }
}

impl Deref for Templ {
    type Target = Map;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}
