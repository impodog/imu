use crate::prelude::*;
use imuc_lexer::token::{Keyword, Symbol};

pub struct LetRule;

impl Rule for LetRule {
    type Output = bind::Let;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Keyword(Keyword::Let))?.is_some() {
            let pat = rules::PatRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Pat".to_owned(),
                    after: TokenKind::Keyword(Keyword::Let),
                })
            })?;

            parser.next_expected(&TokenKind::Symbol(Symbol::Assign))?;

            let val = rules::ExprRule { end: () }.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Expr".to_owned(),
                    after: TokenKind::Symbol(Symbol::Assign),
                })
            })?;

            Ok(Some(bind::Let { pat, val }))
        } else {
            Ok(None)
        }
    }
}
