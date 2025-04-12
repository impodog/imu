use imuc_derive::Spanned;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

/// A immutable, clonable handle of [`PatInner`], holding the pattern info
#[derive(Clone, Spanned)]
pub struct Pat {
    inner: Arc<PatInner>,
    pub span: imuc_lexer::Span,
}

impl Pat {
    pub fn new(pat: PatInner, span: imuc_lexer::Span) -> Self {
        Self {
            inner: Arc::new(pat),
            span,
        }
    }
}

impl Deref for Pat {
    type Target = PatInner;
    fn deref(&self) -> &Self::Target {
        self.inner.deref()
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
pub struct NamedPat(pub BTreeMap<imuc_lexer::StrRef, Option<Type>>);

/// An enumeration used in [`IdentPat`] for an unused or normal name
pub enum IdentKind {
    Unused,
    Value(imuc_lexer::StrRef),
}

/// The flags of type pattern
#[derive(PartialEq, Eq)]
pub enum PatFlags {
    Unique,
    Shared,
    Stack,
}

/// Used in pattern matching, indicating the specific type to match against
#[derive(Spanned)]
pub struct Type {
    pub flags: PatFlags,
    pub kind: TypeKind,
    pub span: imuc_lexer::Span,
}

/// A part of [`Type`] storing only its name and template args
pub enum TypeKind {
    Wildcard,
    Res(imuc_lexer::token::ResTy),
    Single(imuc_lexer::StrRef),
}
