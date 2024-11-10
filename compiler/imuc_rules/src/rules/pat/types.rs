use crate::prelude::*;

pub struct TypeRule;

impl Rule for TypeRule {
    type Output = pat::Type;

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!()
    }
}
