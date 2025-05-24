use crate::prelude::*;

pub struct ModuleRules;

impl Rule for ModuleRules {
    type Output = module::Module;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let items = {
            let mut items = Vec::new();
            while let Some(item) = rules::ItemRule.parse(parser)? {
                items.push(item);
            }
            items
        };
        Ok(Some(module::Module { items }))
    }
}
