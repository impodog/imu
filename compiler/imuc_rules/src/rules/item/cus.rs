use crate::prelude::*;

pub struct CusRule;

impl Rule for CusRule {
    type Output = item::Cus;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let elem = rules::PatRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "pat".to_owned(),
                context: "custom elements".to_owned(),
            })
        })?;
        Ok(Some(item::Cus { elem }))
    }
}
