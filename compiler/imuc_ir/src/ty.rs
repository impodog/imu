use crate::mem::Memory;
use crate::prelude::*;
use imuc_error::*;
use imuc_lexer::token::ResTy;
use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::RwLock;

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
    self_ty: Option<Ty>,
    map: BTreeMap<Ty, (TyInner, RwLock<Option<crate::mem::Memory>>)>,
}

impl Default for TyMap {
    fn default() -> Self {
        Self {
            cur: NonZeroU64::new(1).unwrap(),
            self_ty: None,
            map: Default::default(),
        }
    }
}

impl TyMap {
    /// Inserts a new [`TyInner`] into the map, returning its handle
    pub fn insert(&mut self, inner: TyInner) -> Ty {
        let result = Ty::from(self.cur);
        self.map.insert(result, (inner, RwLock::new(None)));
        self.cur = self
            .cur
            .checked_add(1)
            .expect("ty handle position overflow");
        result
    }

    /// Gets the corresponding [`TyInner`] stored in map
    pub fn get_inner(&self, ty: Ty) -> Option<&TyInner> {
        self.map.get(&ty).map(|data| &data.0)
    }

    /// Gets the corresponding [`Memory`](`crate::mem::Memory`) stored in map
    ///
    /// If the memory layout hasn't been compiled yet, this does the compilation
    ///
    /// During compilation, if the type is recursive with infinite size, an error is returned
    pub fn get_memory<F, R>(&self, ty: Ty, f: F) -> Result<R>
    where
        F: FnOnce(&Memory) -> R,
    {
        if let Some(memory_rwlock) = self.map.get(&ty).map(|data| &data.1) {
            let mut memory_lock = memory_rwlock.try_write().or_else(|_err| {
                let ty = self
                    .get_inner(ty)
                    .expect("inner should exist since memory exists");
                Err(errors::MemoryError::Recursive(ty.name.to_string()))
            })?;
            if memory_lock.is_some() {
                std::mem::drop(memory_lock);
                // If exists, return the memory
                Ok(f(memory_rwlock
                    .read()
                    .expect("lock should not fail")
                    .as_ref()
                    .expect("option check should not fail")))
            } else {
                // Or, compile according to the type layout
                let ty = self
                    .get_inner(ty)
                    .expect("inner should exist since memory exists");
                let memory = match ty.kind {
                    TyKind::Res(res) => match crate::mem::Bytes::try_from(res) {
                        Ok(bytes) => crate::mem::Memory::Bytes(bytes),
                        Err(_) => {
                            if let Some(self_ty) = self.self_ty {
                                return self.get_memory(self_ty, f);
                            } else {
                                return Err(errors::MemoryError::UnexpectedSelf.into());
                            }
                        }
                    },
                    TyKind::Cus(ref cus) => {
                        let mut memory = Memory::elements();
                        for templ in cus.templ.values() {
                            if let Some(ty) = templ.ty {
                                let inner_memory = self.get_memory(ty, Clone::clone)?;
                                memory.push(inner_memory);
                            } else {
                                memory.push(Memory::Templ(templ.id));
                            }
                        }
                        match cus.kind {
                            CusKind::Tuple(ref vec) => {
                                for ty in vec.iter() {
                                    let inner_memory = self.get_memory(*ty, Clone::clone)?;
                                    memory.push(inner_memory);
                                }
                            }
                            CusKind::Struct(ref map) => {
                                for ty in map.values() {
                                    let inner_memory = self.get_memory(*ty, Clone::clone)?;
                                    memory.push(inner_memory);
                                }
                            }
                        }
                        memory
                    }
                };
                *memory_lock = Some(memory);
                std::mem::drop(memory_lock);

                Ok(f(memory_rwlock
                    .read()
                    .expect("lock should not fail")
                    .as_ref()
                    .expect("option check should not fail")))
            }
        } else {
            Err(errors::MemoryError::UnknownHandle(ty.0.to_string()).into())
        }
    }
}

/// The inner data for types
pub struct TyInner {
    name: StrRef,
    kind: TyKind,
}

impl TyInner {
    pub fn new(name: StrRef, kind: TyKind) -> Self {
        Self { name, kind }
    }
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
