use std::borrow::Borrow;
use std::collections::HashSet;
use std::ops::Deref;
use std::sync::Arc;

/// A slightly cheaper clonable reference handle to a string
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StrRef(Arc<String>);
impl ToString for StrRef {
    fn to_string(&self) -> String {
        self.0.as_ref().to_owned()
    }
}

impl Borrow<str> for StrRef {
    fn borrow(&self) -> &str {
        &*self
    }
}

impl Deref for StrRef {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for StrRef
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(Arc::new(value.into()))
    }
}

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

    /// Removes a string referrence from the set. Returns whether the value is present in the set
    pub fn remove(&mut self, s: &str) -> bool {
        self.set.remove(s)
    }
}
