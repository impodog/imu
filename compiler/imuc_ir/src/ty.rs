use crate::prelude::*;
use imuc_lexer::token::ResTy;
use std::collections::BTreeMap;
use std::num::NonZeroU64;

/// A non-zero type handle pointing at a [`TyInner`]
///
/// This can be used as [`Option<Ty>`] without memory overhead
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ty(NonZeroU64);
impl From<NonZeroU64> for Ty {
    fn from(value: NonZeroU64) -> Self {
        Self(value)
    }
}

pub struct TyMap {
    cur: NonZeroU64,
    map: BTreeMap<Ty, TyInner>,
}

impl Default for TyMap {
    fn default() -> Self {
        Self {
            cur: NonZeroU64::new(1).unwrap(),
            map: Default::default(),
        }
    }
}

impl TyMap {
    /// Inserts a new [`TyInner`] into the map, returning its handle
    pub fn insert(&mut self, inner: TyInner) -> Ty {
        let result = Ty::from(self.cur);
        self.map.insert(result, inner);
        self.cur = self
            .cur
            .checked_add(1)
            .expect("ty handle position overflow");
        result
    }

    /// Gets the corresponding [`TyInner`] stored in map
    pub fn get(&self, ty: Ty) -> Option<&TyInner> {
        self.map.get(&ty)
    }
}

/// The inner data for types
pub struct TyInner {
    pub name: StrRef,
    pub kind: TyKind,
}

pub enum TyKind {
    Res(ResTy),
    Cus(Cus),
}

/// Member data of [`TyKind`], defining a custom type with template arguments
///
/// The template arguments are set to None
pub struct Cus {
    pub templ: crate::templ::Templ,
    pub kind: CusKind,
}

/// Member data of [`Cus`], either tuple or structure
pub enum CusKind {
    Tuple(Vec<Ty>),
    Struct(BTreeMap<StrRef, Ty>),
}
