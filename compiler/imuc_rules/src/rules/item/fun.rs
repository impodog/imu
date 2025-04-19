use crate::prelude::*;
use imuc_lexer::token::Symbol;

pub struct FunRule;

impl Rule for FunRule {
    type Output = item::Fun;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let args = rules::PatRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Pat".to_owned(),
                context: "function arguments".to_owned(),
            })
        })?;
        let ret = if parser.next_if(&TokenKind::Symbol(Symbol::Arrow))?.is_some() {
            rules::TypeRule.parse(parser)?
        } else {
            None
        };
        let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Body".to_owned(),
                context: "function body".to_owned(),
            })
        })?;
        Ok(Some(item::Fun {
            param: args,
            body,
            ret,
        }))
    }
}
