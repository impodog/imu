pub(crate) use crate::rules;
pub(crate) use crate::{filtered, lazy_tokens};
pub(crate) use imuc_ast::*;
pub(crate) use imuc_error::*;
pub(crate) use imuc_lexer::TokenKind;
pub(crate) use imuc_parser::{Parser, ParserSequence, Rule};

#[macro_export]
macro_rules! lazy_tokens {
    ($name: ident, $var:ident::$sub:ident) => {
        struct $name;
        impl $name {
            const TOKEN: TokenKind = TokenKind::$var($var::$sub);
        }
        impl<'a> imuc_parser::TokenKindSet<'a> for $name {
            type Iter = std::iter::Once<&'a TokenKind>;
            fn contains(&'a self, token: &TokenKind) -> bool {
                *token == Self::TOKEN
            }
            fn to_iter(&'a self) -> Self::Iter {
                std::iter::once(&Self::TOKEN)
            }
        }
    };
    ($name: ident,  $($var:ident::$sub:ident),+) => {
        struct $name;
        impl<'a> imuc_parser::TokenKindSet<'a> for $name {
            type Iter = std::slice::Iter<'a, TokenKind>;
            fn contains(&'a self, token: &TokenKind) -> bool {
                matches!(token, $(TokenKind::$var($var::$sub))|+)
            }
            fn to_iter(&'a self) -> Self::Iter {
                [$(TokenKind::$var($var::$sub)),+].iter()
            }
        }
    };
    ($name: ident, $($var:ident::$sub:ident),+ and $($pre: ident),+) => {
        struct $name;
        impl<'a> imuc_parser::TokenKindSet<'a> for $name {
            type Iter = std::slice::Iter<'a, TokenKind>;
            fn contains(&'a self, token: &TokenKind) -> bool {
                matches!(token, $(TokenKind::$pre)|+ | $(TokenKind::$var($var::$sub))|+)
            }
            fn to_iter(&'a self) -> Self::Iter {
                [$(TokenKind::$pre),+ , $(TokenKind::$var($var::$sub)),+].iter()
            }
        }
    };
}
#[macro_export]
macro_rules! filtered {
    () => {
        unreachable!("the tokens should be filtered")
    };
    ($by: literal) => {
        unreachable!("the tokens should be filtered by {}", $by)
    };
}
