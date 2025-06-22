use crate::prelude::*;
use imuc_ast::name::Prefix;
use imuc_lexer::token::{Ident, Symbol};

/// Outputs the prefix and the value/type name after the prefix
pub struct PrefixRule;

lazy_tokens!(AfterPrefixTokens, Ident::Value, Ident::Type, Ident::Unused);

impl Rule for PrefixRule {
    type Output = (Prefix, TokenKind, StrRef);
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let mut prefix = Vec::new();
        loop {
            if let Some(value) = parser.next_if(&TokenKind::Ident(Ident::Value))? {
                if parser
                    .next_if(&TokenKind::Symbol(Symbol::DblColon))?
                    .is_some()
                {
                    prefix.push(parser.look_up.insert(value.value));
                } else {
                    return Ok(Some((
                        Prefix::new(prefix),
                        value.kind,
                        parser.look_up.insert(value.value),
                    )));
                }
            } else if let Some(value) = parser.next_if(&AfterPrefixTokens)? {
                return Ok(Some((
                    Prefix::new(prefix),
                    value.kind,
                    parser.look_up.insert(value.value),
                )));
            } else if prefix.is_empty() {
                // No tokens are consumed, and nothing is matched
                return Ok(None);
            } else {
                return Err(errors::SyntaxError::ExpectedAfter {
                    expect: "Name".to_owned(),
                    after: TokenKind::Symbol(Symbol::DblColon),
                }
                .into());
            }
        }
    }
}
