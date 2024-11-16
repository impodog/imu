use crate::prelude::*;
use imuc_lexer::token::BinOp;

pub struct PatRule;

impl Rule for PatRule {
    type Output = pat::Pat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let first = if let Some(first) = rules::TuplePatRule.parse(parser)? {
            pat::Pat::Tuple(first)
        } else if let Some(first) = rules::IdentPatRule.parse(parser)? {
            pat::Pat::Ident(first)
        } else {
            return Ok(None);
        };
        if parser.next_if(&TokenKind::BinOp(BinOp::Or))?.is_some() {
            let pat = rules::AnyPatRule { list: vec![first] }
                .parse(parser)?
                .ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedAfter {
                        expect: "Pat".to_owned(),
                        after: TokenKind::BinOp(BinOp::Or),
                    })
                })?;
            Ok(Some(pat::Pat::Any(pat)))
        } else {
            Ok(Some(first))
        }
    }
}
