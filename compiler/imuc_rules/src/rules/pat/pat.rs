use crate::prelude::*;
use imuc_lexer::token::BinOp;

pub struct PatRule;

impl Rule for PatRule {
    type Output = pat::Pat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        let first = if let Some(named) = rules::CusPatRule.parse(parser)? {
            pat::PatInner::Cus(named)
        } else if let Some(first) = rules::TuplePatRule.parse(parser)? {
            // NOTE: Tuple pat returns `PatInner` because it can be without comma
            first
        } else if let Some(first) = rules::IdentPatRule.parse(parser)? {
            pat::PatInner::Ident(first)
        } else {
            return Ok(None);
        };
        if parser.next_if(&TokenKind::BinOp(BinOp::Or))?.is_some() {
            let pat = rules::AnyPatRule {
                list: vec![pat::Pat::new(
                    first,
                    parser.file_info().into_span(cursor_begin),
                )],
            }
            .parse(parser)?
            .ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Pat".to_owned(),
                    after: TokenKind::BinOp(BinOp::Or),
                })
            })?;
            Ok(Some(pat::Pat::new(
                pat::PatInner::Any(pat),
                parser.file_info().into_span(cursor_begin),
            )))
        } else {
            Ok(Some(pat::Pat::new(
                first,
                parser.file_info().into_span(cursor_begin),
            )))
        }
    }
}
