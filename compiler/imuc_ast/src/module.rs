use crate::item::Item;

/// The syntax tree entry point for modules
pub struct Module {
    pub import: Vec<Import>,
    pub items: Vec<Item>,
}

/// The level of publicity in item definitions and items
pub enum Public {
    Pub,
    Module,
    Priv,
}

/// A single import from the module
pub struct Import {
    pub public: Public,
    pub name: Vec<String>,
}
