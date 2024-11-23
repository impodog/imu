use crate::prelude::*;
use imuc_lexer::token::{Ident, Pair, Symbol};
use std::collections::BTreeMap;

pub struct StructExprRule;

impl Rule for StructExprRule {
    type Output = expr::Struct;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if let Some(ty) = rules::TypeRule.parse(parser)? {
            parser.next_expected(&TokenKind::Pair(Pair::LeftParen))?;
            let mut elem = BTreeMap::new();
            let mut comma = false;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightParen))?
                    .is_some()
                {
                    break;
                } else if !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                let name = parser.next_expected(&TokenKind::Ident(Ident::Value))?;

                parser.next_expected(&TokenKind::Symbol(Symbol::Colon))?;

                let expr = rules::ExprRule {
                    end: TokenKind::Symbol(Symbol::Comma),
                }
                .parse(parser)?
                .ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Expr".to_owned(),
                        context: "tuple expression".to_owned(),
                    })
                })?;

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                elem.insert(parser.look_up.insert(name.value), expr);
            }
            Ok(Some(expr::Struct { ty, elem }))
        } else {
            Ok(None)
        }
    }
}
