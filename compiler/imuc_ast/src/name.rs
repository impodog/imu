use imuc_lexer::StrRef;
use std::collections::HashSet;

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
