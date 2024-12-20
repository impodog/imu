use crate::graph::Dag;
use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use sym::{ty, Ty};

type TyMap = HashMap<StrRef, Ty>;

#[derive(Default)]
pub struct Types {
    map: TyMap,
}

impl Deref for Types {
    type Target = TyMap;
    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl Types {
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
        let mut map = HashMap::<StrRef, (usize, Option<Ty>)>::new();
        let mut graph = Dag::new();
        let add_edge = |graph: &mut Dag<StrRef>,
                        map: &mut HashMap<StrRef, (usize, Option<Ty>)>,
                        node: usize,
                        item: &ty::TyItem| {
            match item {
                ty::TyItem::Solid(_) => {}
                ty::TyItem::Pending(dep) => {
                    if self.get(dep).is_some() {
                    } else if let Some(dep) = map.get_mut(dep) {
                        graph.add(dep.0, node);
                    } else {
                        let index = graph.push(dep.clone());
                        map.insert(dep.clone(), (index, None));
                        graph.add(index, node);
                    }
                }
            }
        };
        let modify_item = |map: &TyMap, item: &ty::TyItem| -> Result<ty::TyItem> {
            match item {
                ty::TyItem::Solid(_) => Ok(item.clone()),
                ty::TyItem::Pending(pending) => map
                    .get(pending)
                    .ok_or_else(|| errors::IrError::LoopedReference.into())
                    .cloned()
                    .map(ty::TyItem::Solid),
            }
        };
        for (name, ty) in iter {
            let node = if let Some(node) = map.get_mut(name) {
                if node.1.is_none() {
                    node.1 = Some(ty.clone());
                }
                node.0
            } else {
                let index = graph.push(name.clone());
                map.insert(name.clone(), (index, Some(ty.clone())));
                index
            };
            match &ty.kind {
                ty::TyKind::Res(_) | ty::TyKind::Ptr(_) => {}
                ty::TyKind::Ref(item) => {
                    add_edge(&mut graph, &mut map, node, item);
                }
                ty::TyKind::Tuple(tuple) => {
                    for item in tuple.0.iter() {
                        add_edge(&mut graph, &mut map, node, item);
                    }
                }
                ty::TyKind::Struct(cus) => {
                    for item in cus.0.values() {
                        add_edge(&mut graph, &mut map, node, item);
                    }
                }
            }
        }

        // Extract values using the topo sort order
        let order = graph.topo_sort().ok_or(errors::IrError::LoopedReference)?;
        for name in order.into_iter() {
            let ty = map
                .remove(&name)
                .and_then(|(_, ty)| ty)
                .ok_or_else(|| errors::IrError::NoSuchType(name.to_string()))?;
            let kind = match &ty.kind {
                // pointers does not resolve recursively
                ty::TyKind::Ptr(item) => ty::TyKind::Ptr(item.clone()),
                ty::TyKind::Ref(item) => ty::TyKind::Ref(modify_item(&self.map, item)?),
                ty::TyKind::Res(res) => ty::TyKind::Res(*res),
                ty::TyKind::Tuple(tuple) => {
                    let mut value = Vec::new();
                    for item in tuple.0.iter() {
                        value.push(modify_item(&self.map, item)?);
                    }
                    ty::TyKind::Tuple(ty::Tuple(value))
                }
                ty::TyKind::Struct(cus) => {
                    let mut value = BTreeMap::new();
                    for (name, item) in cus.0.iter() {
                        value.insert(name.clone(), modify_item(&self.map, item)?);
                    }
                    ty::TyKind::Struct(ty::Struct(value))
                }
            };
            let ty = ty::Ty::new(ty::TyInner {
                name: ty.name.clone(),
                kind,
                external: ty.external,
            });
            self.map.insert(ty.name.clone(), ty);
        }
        Ok(())
    }

    pub fn insert(&mut self, name: StrRef, ty: Ty) {
        self.map.insert(name, ty);
    }
}
