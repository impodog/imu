use crate::graph::Dag;
use crate::prelude::*;
use imuc_error::errors::ctx::{ConvError, Severity};
use std::collections::{BTreeMap, HashMap};
use std::ops::Deref;
use sym::{ty, Ty};

type TyMap = HashMap<StrRef, Ty>;

/// A direct representation of strings mapped to types,
/// can be used to convert `ty::TyItem`] into [`Ty`
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

    /// Resolves the `TyItem`](`ty::TyItem`) into a direct [`Ty`, if possible
    pub fn resolve<'a, 'b>(&'a self, item: &'b ty::TyItem) -> Option<&'a Ty>
    where
        'b: 'a,
    {
        match item {
            ty::TyItem::Solid(ty) => Some(ty),
            ty::TyItem::Pending(key) => self.get(key),
        }
    }

    /// Resolves the `TyItem`](`ty::TyItem`) into a direct [`Ty`, if possible,
    /// or an error with the given span and a relatively fixed message is returned
    ///
    /// If you want custom messages, use `Self::resolve` instead
    pub fn resolve_or<'a, 'b>(
        &'a self,
        item: &'b ty::TyItem,
        span: imuc_lexer::Span,
    ) -> Result<&'a Ty, ConvError>
    where
        'b: 'a,
    {
        self.resolve(item).ok_or_else(move || {
            ConvError::new(Severity::Error, span).with_text(
                "Ty is undefined",
                format!("Unable to find ty {}", item.name()),
            )
        })
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
                ty::TyKind::Res(_) | ty::TyKind::Ptr(_) | ty::TyKind::Fun { .. } => {}
                ty::TyKind::Ref(item) => {
                    add_edge(&mut graph, &mut map, node, item);
                }
                ty::TyKind::Tuple(tuple) => {
                    for item in tuple.0.iter() {
                        add_edge(&mut graph, &mut map, node, item);
                    }
                }
                ty::TyKind::Cus(cus) => {
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
                ty::TyKind::Fun { param, ret } => ty::TyKind::Fun {
                    param: param.clone(),
                    ret: ret.clone(),
                },
                ty::TyKind::Res(res) => ty::TyKind::Res(*res),
                ty::TyKind::Tuple(tuple) => {
                    let mut value = Vec::new();
                    for item in tuple.0.iter() {
                        value.push(modify_item(&self.map, item)?);
                    }
                    ty::TyKind::Tuple(ty::Tuple(value))
                }
                ty::TyKind::Cus(cus) => {
                    let mut value = BTreeMap::new();
                    for (name, item) in cus.0.iter() {
                        value.insert(name.clone(), modify_item(&self.map, item)?);
                    }
                    ty::TyKind::Cus(ty::Cus(value))
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

    pub fn insert(&mut self, ty: Ty) {
        self.map.insert(ty.name.clone(), ty);
    }

    pub fn or_insert_with<F>(&mut self, name: StrRef, f: F) -> &mut Ty
    where
        F: FnOnce() -> Ty,
    {
        self.map.entry(name).or_insert_with(f)
    }

    /// Extracts local types from the type map, and store them in a `BTreeMap` suitable for
    /// ir module representation.
    pub fn extract_map(&self) -> BTreeMap<StrRef, Ty> {
        let mut map = BTreeMap::new();
        for (name, ty) in self.iter() {
            if !ty.external {
                map.insert(name.clone(), ty.clone());
            }
        }
        map
    }
}
