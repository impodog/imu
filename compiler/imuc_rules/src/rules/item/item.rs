use crate::prelude::*;
use imuc_lexer::token::{Ident, Keyword};

lazy_tokens!(
    ItemTokens,
    Keyword::Fun,
    Keyword::Cus,
    Keyword::For,
    Keyword::Val
);

lazy_tokens!(ValueTokens, Ident::Value, Ident::Unused);
lazy_tokens!(TypeTokens, Ident::Type, Ident::Unused);

pub struct ItemRule;

impl Rule for ItemRule {
    type Output = item::Item;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let public = rules::PublicRule
            .parse(parser)?
            .expect("publicity rule should not return None");
        let input = parser.next_if(&ItemTokens)?;
        if let Some(input) = input {
            let templ = rules::TemplDefRule.parse(parser)?.unwrap_or_default();
            match input.kind {
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Fun => {
                        let name = parser.next_expected(&ValueTokens)?;
                        let fun = rules::FunRule.parse(parser)?.ok_or_else(|| {
                            parser.map_err(errors::SyntaxError::ExpectedIn {
                                expect: "Fun".to_owned(),
                                context: "function defintion".to_owned(),
                            })
                        })?;
                        Ok(Some(item::Item {
                            public,
                            templ,
                            name: parser.look_up.insert(&name.value),
                            kind: item::ItemKind::Fun(fun),
                        }))
                    }
                    Keyword::Cus => {
                        let name = parser.next_expected(&TypeTokens)?;
                        let cus = rules::CusRule.parse(parser)?.ok_or_else(|| {
                            parser.map_err(errors::SyntaxError::ExpectedIn {
                                expect: "Cus".to_owned(),
                                context: "custom defintion".to_owned(),
                            })
                        })?;
                        Ok(Some(item::Item {
                            public,
                            templ,
                            name: parser.look_up.insert(name.value),
                            kind: item::ItemKind::Cus(cus),
                        }))
                    }
                    Keyword::For => todo!(),
                    Keyword::Val => todo!(),
                    _ => filtered!(),
                },
                _ => filtered!(),
            }
        } else if let module::Public::Priv = public {
            // No publicity keywords matched, thus the rule is not matched
            Ok(None)
        } else {
            // Or, a publicity keyword is not followed by an item. This is an error
            parser.error(errors::SyntaxError::ExpectedAfter {
                expect: "Item".to_owned(),
                after: TokenKind::Keyword(Keyword::Pub),
            })
        }
    }
}
