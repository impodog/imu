use imuc_lexer::StrRef;
use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

/// A name lookup set that produces [`StrRef`] on insertion
///
/// This reduces the times of cloning the same name reference
#[derive(Default)]
pub struct LookUp {
    set: HashSet<StrRef>,
}

impl LookUp {
    /// Inserts a new string reference to the set, returning its corresponding handle
    pub fn insert(&mut self, s: &str) -> StrRef {
        if let Some(value) = self.set.get(s) {
            value.clone()
        } else {
            let value = StrRef::from(s);
            self.set.insert(value.clone());
            value
        }
    }

    /// Removes a string reference from the set. Returns whether the value is present in the set
    pub fn remove(&mut self, s: &str) -> bool {
        self.set.remove(s)
    }
}

/// The first item of the prefix, supporting "loc" keyword
#[derive(Debug, Clone)]
pub enum PrefixFirst {
    Name(StrRef),
    Loc,
}
/// Prefix connected by double colons used before values and types to specify the namespace
/// Its iterator only iterates over items after the first name of the prefix
#[derive(Debug, Clone)]
pub struct Prefix {
    pub first: PrefixFirst,
    pub remain: Vec<StrRef>,
}
impl Deref for Prefix {
    type Target = Vec<StrRef>;
    fn deref(&self) -> &Self::Target {
        &self.remain
    }
}
impl DerefMut for Prefix {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.remain
    }
}
impl Prefix {
    /// Creates a new prefix with the first element
    pub fn new(first: PrefixFirst) -> Self {
        Self {
            first,
            remain: Default::default(),
        }
    }
}

/// A compound type with both the namespace prefix and the name
#[derive(Debug, Clone)]
pub struct PrefixedName {
    pub prefix: Prefix,
    pub name: StrRef,
}

impl PrefixedName {
    /// Creates a new prefixed name
    pub fn new(prefix: Prefix, name: StrRef) -> Self {
        Self { prefix, name }
    }
}
