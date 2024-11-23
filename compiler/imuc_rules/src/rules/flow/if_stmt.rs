use crate::prelude::*;
use imuc_lexer::token::{Keyword, Pair};

pub struct IfRule;

impl Rule for IfRule {
    type Output = flow::If;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Keyword(Keyword::If))?.is_some() {
            let cond = rules::ExprRule {
                end: TokenKind::Pair(Pair::LeftBrace),
            }
            .parse(parser)?
            .ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Expr".to_owned(),
                    after: TokenKind::Keyword(Keyword::If),
                })
            })?;
            let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedIn {
                    expect: "Body".to_owned(),
                    context: "if statement".to_owned(),
                })
            })?;
            Ok(Some(flow::If {
                cond: Box::new(cond),
                body,
            }))
        } else {
            Ok(None)
        }
    }
}
