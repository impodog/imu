use crate::prelude::*;

pub struct BindRule;

impl Rule for BindRule {
    type Output = bind::Bind;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if let Some(bind) = rules::LetRule.parse(parser)? {
            Ok(Some(bind::Bind::Let(bind)))
        } else if let Some(item) = rules::ItemRule.parse(parser)? {
            Ok(Some(bind::Bind::Item(item)))
        } else {
            Ok(None)
        }
    }
}
