use crate::prelude::*;

pub struct BodyRule;

impl Rule for BodyRule {
    type Output = expr::Body;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!();
    }
}
