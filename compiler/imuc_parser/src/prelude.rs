pub(crate) use crate::{Parser, Rule};
pub(crate) use imuc_ast::*;
pub(crate) use imuc_error::*;
pub(crate) use imuc_lexer::TokenKind;
pub(crate) use lazy_static::lazy_static;
pub(crate) use std::collections::BTreeSet;

#[macro_export]
macro_rules! lazy_tokens {
    ($name: ident, $tokens: expr) => {
        lazy_static! {
            static ref $name: BTreeSet<TokenKind> = BTreeSet::from_iter($tokens);
        }
    };
}
