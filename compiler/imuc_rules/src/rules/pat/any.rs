use crate::prelude::*;
use imuc_lexer::token::BinOp;

/// Only invoked when a '|' symbol exists after a matched pattern,
/// and the first element of the pattern must be given
pub struct AnyPatRule {
    pub list: Vec<pat::Pat>,
}

impl Rule for AnyPatRule {
    type Output = pat::AnyPat;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let mut list = self.list;
        loop {
            let pat = rules::PatRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Pat".to_owned(),
                    after: TokenKind::BinOp(BinOp::Or),
                })
            })?;
            list.push(pat);
            if parser.next_if(&TokenKind::BinOp(BinOp::Or))?.is_none() {
                break;
            }
        }
        Ok(Some(pat::AnyPat(list)))
    }
}
