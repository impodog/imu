use crate::prelude::*;
use imuc_lexer::token::Pair;

pub struct TuplePatRule;

impl Rule for TuplePatRule {
    type Output = pat::TuplePat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftParen))?.is_some() {
            let mut list = Vec::new();
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightParen))?
                    .is_some()
                {
                    break;
                }
                let pat = rules::PatRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Pat".to_owned(),
                        context: "tuple pattern".to_owned(),
                    })
                })?;
                list.push(pat);
            }
            Ok(Some(pat::TuplePat(list)))
        } else {
            Ok(None)
        }
    }
}
