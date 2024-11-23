use crate::prelude::*;
use imuc_lexer::token::{Pair, Symbol};

pub struct TupleExprRule;

impl Rule for TupleExprRule {
    type Output = expr::Tuple;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftParen))?.is_some() {
            let mut elem = Vec::new();
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

                elem.push(expr);
            }
            Ok(Some(expr::Tuple { elem }))
        } else {
            Ok(None)
        }
    }
}
