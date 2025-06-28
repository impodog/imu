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

/// Prefix connected by double colons used before values and types to specify the namespace
#[derive(Default, Debug, Clone)]
pub struct Prefix(pub Vec<StrRef>);

impl Prefix {
    pub fn new(prefix: Vec<StrRef>) -> Self {
        Self(prefix)
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

    /// Creates a name pointing to local variables (aka without prefix)
    pub fn local(name: StrRef) -> Self {
        Self {
            prefix: Default::default(),
            name,
        }
    }
}
