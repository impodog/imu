use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

/// A immutable, clonable handle of [`PatInner`], holding the pattern info
#[derive(Clone)]
pub struct Pat(Arc<PatInner>);

impl Pat {
    pub fn new(pat: PatInner) -> Self {
        Self(Arc::new(pat))
    }
}

impl Deref for Pat {
    type Target = PatInner;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

/// A pattern to be matched against values
pub enum PatInner {
    Ident(IdentPat),
    Tuple(TuplePat),
    Any(AnyPat),
    Named(NamedPat),
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

/// A name group of patterns that can be matched according to names
pub struct NamedPat(pub BTreeMap<crate::StrRef, Option<Type>>);

/// An enumeration used in [`IdentPat`] for an unused or normal name
pub enum IdentKind {
    Unused,
    Value(crate::StrRef),
}

/// The flags of type pattern
#[derive(PartialEq, Eq)]
pub enum PatFlags {
    Unique,
    Shared,
    Stack,
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
    Single(crate::StrRef),
}
