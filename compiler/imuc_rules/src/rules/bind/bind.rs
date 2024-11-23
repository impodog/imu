use crate::prelude::*;

pub struct BindRule;

impl Rule for BindRule {
    type Output = bind::Bind;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!()
    }
}
