use crate::prelude::*;
use imuc_lexer::token::Keyword;

lazy_tokens!(PublicTokens, Keyword::Pub);

pub struct PublicRule;

impl Rule for PublicRule {
    type Output = module::Public;

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&PublicTokens)?;
        if let Some(input) = input {
            match input.kind {
                TokenKind::Keyword(keyword) => match keyword {
                    Keyword::Pub => Ok(Some(module::Public::Pub)),
                    _ => filtered!(),
                },
                _ => filtered!(),
            }
        } else {
            Ok(Some(module::Public::Priv))
        }
    }
}
