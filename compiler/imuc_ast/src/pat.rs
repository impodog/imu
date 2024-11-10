/// A pattern to be matched against values
pub enum Pat {
    Ident(IdentPat),
    Tuple(TuplePat),
    Any(AnyPat),
}

/// The basic pattern, matching value to a certain type
pub struct IdentPat {
    pub ident: IdentKind,
    pub ty: Option<Type>,
}

/// A linear group of patterns
pub struct TuplePat(pub Vec<Pat>);

/// A tree-like group of patterns, matching one of them
pub struct AnyPat(pub Vec<Pat>);

/// An enumeration used in [`IdentPat`] for an unused or normal name
pub enum IdentKind {
    Unused,
    Value(String),
}

/// The flags of type pattern
pub enum PatFlags {
    Unique,
    Shared,
}

/// Used in pattern matching, indicating the specific type to match against
pub struct Type {
    pub flags: PatFlags,
    pub kind: TypeKind,
}

/// A part of [`Type`] storing only its name and template args
pub enum TypeKind {
    Wildcard,
    Res(imuc_lexer::token::ResTy),
    Single(String),
    Template(String, Vec<TypeKind>),
}
