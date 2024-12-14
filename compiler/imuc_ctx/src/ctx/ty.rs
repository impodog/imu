use crate::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;
use sym::{ty, Ty};

type TyMap = HashMap<StrRef, Ty>;

pub struct TyCtx {
    map: TyMap,
}

impl Deref for TyCtx {
    type Target = TyMap;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl TyCtx {
    pub fn get(&self, key: &str) -> Option<&Ty> {
        self.map.get(key)
    }

    /// Merge a resolvable list of types
    ///
    /// If the iterator contains loops or undefined references, the function returns an error
    pub fn merge<'a, I>(&mut self, iter: I) -> Result<()>
    where
        I: IntoIterator<Item = (&'a StrRef, &'a Ty)>,
    {
        let mut map = HashMap::<StrRef, usize>::new();
        let mut graph = crate::Dag::new();
        for (name, ty) in iter {
            let node = graph.push((name, ty));
            // todo
        }
        Ok(())
    }
}
