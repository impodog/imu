/// A pattern to be matched against values
pub enum Pat {
    Ident(IdentPat),
    Tuple(TuplePat),
    Any(AnyPat),
}

/// The basic pattern, matching value to a certain type
pub struct IdentPat {
    pub ident: String,
    pub ty: Option<Ty>,
}

/// A linear group of patterns
pub struct TuplePat(pub Vec<Pat>);

/// A tree-like group of patterns, matching one of them
pub struct AnyPat(pub Vec<Pat>);

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
    Single(String),
    Template(String, Vec<TypeKind>),
}
