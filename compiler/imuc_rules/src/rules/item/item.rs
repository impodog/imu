use crate::prelude::*;
use imuc_lexer::token::{Ident, Keyword};

lazy_tokens!(
    ItemTokens,
    Keyword::Fun,
    Keyword::Cus,
    Keyword::For,
    Keyword::Val
);

lazy_tokens!(IdentTokens, Ident::Value, Ident::Type, Ident::Unused);

pub struct ItemRule;

impl Rule for ItemRule {
    type Output = item::Item;

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let public = rules::PublicRule
            .parse(parser)?
            .expect("publicity rule should not return None");
        let input = parser.next_if(&ItemTokens)?;
        if let Some(input) = input {
            let templ = rules::TemplRule.parse(parser)?.unwrap_or_default();
            let name = parser.next_expected(&IdentTokens)?;
            match input.kind {
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Fun => todo!(),
                    Keyword::Cus => todo!(),
                    Keyword::For => todo!(),
                    Keyword::Val => todo!(),
                    _ => filtered!(),
                },
                _ => filtered!(),
            }
        } else if let module::Public::Priv = public {
            Ok(None)
        } else {
            parser.error(errors::SyntaxError::ExpectedAfter {
                expect: "Item".to_owned(),
                after: TokenKind::Keyword(Keyword::Pub),
            })
        }
    }
}
