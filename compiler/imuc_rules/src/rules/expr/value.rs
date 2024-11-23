use crate::prelude::*;
use imuc_lexer::token::{Ident, ResVal};

lazy_tokens!(ResValTokens, ResVal::True, ResVal::False, ResVal::SelfValue);

pub struct ValueRule;

impl Rule for ValueRule {
    type Output = expr::Value;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if let Some(input) = parser.next_if(&TokenKind::Ident(Ident::Value))? {
            Ok(Some(expr::Value::Name(parser.look_up.insert(input.value))))
        } else if let Some(_input) = parser.next_if(&TokenKind::Ident(Ident::Unused))? {
            Ok(Some(expr::Value::Unused))
        } else if let Some(input) = parser.next_if(&ResValTokens)? {
            if let TokenKind::ResVal(res) = input.kind {
                Ok(Some(expr::Value::Res(res)))
            } else {
                unreachable!("the token kind should be TokenKind::ResVal")
            }
        } else {
            Ok(None)
        }
    }
}
