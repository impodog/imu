use crate::prelude::*;

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
        let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Body".to_owned(),
                context: "function body".to_owned(),
            })
        })?;
        Ok(Some(item::Fun { args, body }))
    }
}
