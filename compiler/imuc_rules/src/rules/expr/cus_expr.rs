use crate::prelude::*;
use imuc_lexer::token::{Ident, Pair, Symbol};
use std::collections::BTreeMap;

pub struct CusExprRule;

impl Rule for CusExprRule {
    type Output = expr::Cus;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
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
                } else if !elem.is_empty() && !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                // Used by shorthand field init
                let cursor_begin = parser.relative_cursor();

                let name = parser.next_expected(&TokenKind::Ident(Ident::Value))?;

                let expr = if parser.next_if(&TokenKind::Symbol(Symbol::Colon))?.is_some() {
                    rules::ExprRule {
                        end: TokenKind::Symbol(Symbol::Comma),
                    }
                    .parse(parser)?
                    .ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Expr".to_owned(),
                            context: "tuple expression".to_owned(),
                        })
                    })?
                } else {
                    expr::Expr::Value(expr::Value {
                        value: expr::ValueInner::Name(name.value.into()),
                        span: parser.file_info().into_span(cursor_begin),
                    })
                };

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                elem.insert(parser.look_up.insert(name.value), expr);
            }
            Ok(Some(expr::Cus {
                ty,
                elem,
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else {
            Ok(None)
        }
    }
}
