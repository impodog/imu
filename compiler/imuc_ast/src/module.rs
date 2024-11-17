use crate::item::Item;
use std::path::PathBuf;

/// The syntax tree entry point for modules
pub struct Module {
    pub import: Vec<Import>,
    pub items: Vec<Item>,
}

/// The level of publicity in item definitions and items
pub enum Public {
    Pub,
    // TODO: Add keywords corresponding to this, if necessary
    Module,
    Priv,
}

/// An item of the import statement, either a type or a value
pub enum ImportItemKind {
    Value(crate::StrRef),
    Type(crate::StrRef),
}

pub struct ImportItem {
    pub kind: ImportItemKind,
    pub alias: Option<crate::StrRef>,
}

/// A single import from the module
pub struct Import {
    pub file: imuc_path::File,
    pub item: Vec<ImportItem>,
}
