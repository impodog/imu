use crate::item::Item;
use imuc_derive::Spanned;

/// The syntax tree entry point for modules
pub struct Module {
    pub sub_nodes: Vec<SubNode>,
    pub items: Vec<Item>,
}

/// The level of publicity in item definitions and items
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Public {
    #[default]
    Priv,
    // TODO: Add keywords corresponding to this, if necessary
    Module,
    Pub,
}

pub struct SubNode {
    pub name: imuc_lexer::StrRef,
    pub module: Module,
    pub span: imuc_lexer::Span,
}

/// An item of the import statement, either a type or a value
pub enum ImportItemKind {
    Value(imuc_lexer::StrRef),
    Type(imuc_lexer::StrRef),
    Wildcard,
    WildcardWithPrefix,
}

#[derive(Spanned)]
pub struct ImportItem {
    pub prefix: crate::name::Prefix,
    pub kind: ImportItemKind,
    pub alias: Option<imuc_lexer::StrRef>,
    pub span: imuc_lexer::Span,
}

pub enum ImportDir {
    External(std::path::PathBuf),
    Loc,
}

/// A single import from the module
#[derive(Spanned)]
pub struct Import {
    pub dir: ImportDir,
    pub item: Vec<ImportItem>,
    pub span: imuc_lexer::Span,
}
