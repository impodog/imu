use crate::prelude::*;
use imuc_lexer::token::Pair;

pub struct ForRule;

impl Rule for ForRule {
    type Output = item::For;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let ty = rules::TypeRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Type".to_owned(),
                context: "type implementation".to_owned(),
            })
        })?;
        let mut items = Vec::new();
        parser.next_expected(&TokenKind::Pair(Pair::LeftBrace))?;
        loop {
            if parser
                .next_if(&TokenKind::Pair(Pair::RightBrace))?
                .is_some()
            {
                break;
            }
            let item = rules::ItemRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedIn {
                    expect: "Item".to_owned(),
                    context: "type implementation".to_owned(),
                })
            })?;
            items.push(item);
        }

        Ok(Some(item::For { ty, items }))
    }
}
