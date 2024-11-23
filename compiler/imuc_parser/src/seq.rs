use imuc_error::*;
use imuc_lexer::TokenKind;

/// The direct input to parser, holding [`TokenKind`] and its corresponding string slice
#[derive(Debug, Clone, Copy)]
pub struct ParserInput<'s> {
    pub kind: TokenKind,
    pub value: &'s str,
}

/// Trait for types that can be applied to a parser
/// This requires an iterator over [`ParserInput`] and context info
pub trait ParserSequence<'s>: Iterator<Item = ParserInput<'s>> + Send + Sync {
    fn map_error(&self, err: Error) -> Error;
}

/// Any type that holds a series of tokens to be matched against
pub trait TokenKindSet<'a> {
    type Iter: Iterator<Item = &'a TokenKind>;

    fn contains(&'a self, token: &TokenKind) -> bool;

    fn to_iter(&'a self) -> Self::Iter;
}

impl<'a> TokenKindSet<'a> for std::collections::BTreeSet<TokenKind> {
    type Iter = std::collections::btree_set::Iter<'a, TokenKind>;

    fn contains(&'a self, token: &TokenKind) -> bool {
        self.contains(token)
    }

    fn to_iter(&'a self) -> Self::Iter {
        self.iter()
    }
}

impl<'a> TokenKindSet<'a> for TokenKind {
    type Iter = std::iter::Once<&'a TokenKind>;

    fn contains(&'a self, token: &TokenKind) -> bool {
        *self == *token
    }

    fn to_iter(&'a self) -> Self::Iter {
        std::iter::once(self)
    }
}

/// Marks an empty token set
impl<'a> TokenKindSet<'a> for () {
    type Iter = std::iter::Empty<&'a TokenKind>;

    fn contains(&'a self, _token: &TokenKind) -> bool {
        false
    }

    fn to_iter(&'a self) -> Self::Iter {
        std::iter::empty()
    }
}

impl<'a, T1, T2> TokenKindSet<'a> for (T1, T2)
where
    T1: TokenKindSet<'a>,
    T2: TokenKindSet<'a>,
{
    type Iter = std::iter::Chain<T1::Iter, T2::Iter>;

    fn contains(&'a self, token: &TokenKind) -> bool {
        self.0.contains(token) || self.1.contains(token)
    }

    fn to_iter(&'a self) -> Self::Iter {
        self.0.to_iter().chain(self.1.to_iter())
    }
}
