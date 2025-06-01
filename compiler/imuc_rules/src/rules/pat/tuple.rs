use crate::prelude::*;
use imuc_lexer::token::{Pair, Symbol};

pub struct TuplePatRule;

impl Rule for TuplePatRule {
    type Output = pat::PatInner;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftParen))?.is_some() {
            let mut list = Vec::new();
            let mut comma = false;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightParen))?
                    .is_some()
                {
                    break;
                } else if !list.is_empty() && !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                let pat = rules::PatRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Pat".to_owned(),
                        context: "tuple pattern".to_owned(),
                    })
                })?;

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                list.push(pat);
            }
            if list.len() == 1 && !comma {
                let pat = list
                    .pop()
                    .expect("list should contain a value since its len is 1");
                Ok(Some(pat.into_inner()))
            } else {
                // NOTE: This does not consider empty tuples, but the pattern will be extracted
                // as unit by the converter
                Ok(Some(pat::PatInner::Tuple(pat::TuplePat(list))))
            }
        } else {
            Ok(None)
        }
    }
}
