use crate::prelude::*;
use imuc_lexer::token::{Ident, Pair, Symbol};
use std::collections::BTreeMap;

pub struct CusPatRule;

impl Rule for CusPatRule {
    type Output = pat::CusPat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftBrace))?.is_some() {
            let mut list = BTreeMap::new();
            let mut comma = false;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightBrace))?
                    .is_some()
                {
                    break;
                } else if !list.is_empty() && !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightBrace),
                    }));
                }

                let input = parser
                    .next_if(&TokenKind::Ident(Ident::Value))?
                    .ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Ident::Value".to_string(),
                            context: "Cus".to_string(),
                        })
                    })?;
                let ident = parser.look_up.insert(input.value);
                let ty = if parser.next_if(&TokenKind::Symbol(Symbol::Colon))?.is_some() {
                    Some(rules::TypeRule.parse(parser)?.ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedAfter {
                            expect: "Type".to_owned(),
                            after: TokenKind::Symbol(Symbol::Colon),
                        })
                    })?)
                } else {
                    None
                };
                list.insert(ident, ty);

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();
            }
            Ok(Some(pat::CusPat(list)))
        } else {
            Ok(None)
        }
    }
}
