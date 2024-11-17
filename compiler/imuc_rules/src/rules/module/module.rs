use crate::prelude::*;

pub struct ModuleRules;

impl Rule for ModuleRules {
    type Output = module::Module;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        // Consumes imports
        let import = {
            let mut import = Vec::new();
            loop {
                let rule = rules::ImportRule {
                    import: &mut import,
                };
                if rule.parse(parser)?.is_none() {
                    break;
                }
                while parser.next_if(&TokenKind::Semicolon)?.is_some() {}
            }
            import
        };
        let items = {
            let mut items = Vec::new();
            while let Some(item) = rules::ItemRule.parse(parser)? {
                items.push(item);
            }
            items
        };
        Ok(Some(module::Module { import, items }))
    }
}
