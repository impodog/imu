use crate::prelude::*;
use imuc_lexer::token::{Ident, Symbol};

pub struct IdentPatRule;

lazy_tokens!(IdentTokens, Ident::Value, Ident::Unused);

impl Rule for IdentPatRule {
    type Output = pat::IdentPat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&IdentTokens)?;
        if let Some(input) = input {
            let ty = if parser.next_if(&TokenKind::Symbol(Symbol::Colon))?.is_some() {
                let ty = rules::TypeRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedAfter {
                        expect: "Type".to_owned(),
                        after: TokenKind::Symbol(Symbol::Colon),
                    })
                })?;
                Some(ty)
            } else {
                None
            };
            let ident = if let TokenKind::Ident(Ident::Unused) = input.kind {
                pat::IdentKind::Unused
            } else {
                pat::IdentKind::Value(parser.look_up(input.value))
            };
            Ok(Some(pat::IdentPat { ident, ty }))
        } else {
            Ok(None)
        }
    }
}
