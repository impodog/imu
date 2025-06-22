use crate::prelude::*;
use imuc_lexer::token::{Ident, ResVal, Symbol};

lazy_tokens!(ResValTokens, ResVal::True, ResVal::False);

pub struct ValueRule;

impl Rule for ValueRule {
    type Output = expr::Value;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        let value = if let Some(input) = parser.next_if(&TokenKind::Ident(Ident::Value))? {
            expr::ValueInner::Name(parser.look_up.insert(input.value))
        } else if let Some(_input) = parser.next_if(&TokenKind::Ident(Ident::Unused))? {
            expr::ValueInner::Unused
        } else if let Some(input) = parser.next_if(&ResValTokens)? {
            if let TokenKind::ResVal(res) = input.kind {
                expr::ValueInner::Res(res)
            } else {
                unreachable!("the token kind should be TokenKind::ResVal")
            }
        } else {
            return Ok(None);
        };
        Ok(Some(expr::Value {
            value,
            span: parser.file_info().into_span(cursor_begin),
        }))
    }
}
