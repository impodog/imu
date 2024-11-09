use crate::{lazy_tokens, prelude::*};
use imuc_lexer::token::Literal;

lazy_tokens!(
    PRIM_TOKENS,
    [
        TokenKind::Literal(Literal::Integer),
        TokenKind::Literal(Literal::Float),
        TokenKind::Literal(Literal::String),
        TokenKind::Literal(Literal::MultiString),
    ]
);

pub struct PrimRule;

impl Rule for PrimRule {
    type Output = prim::Prim;

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: crate::ParserSequence<'s>,
    {
        let input = parser.next_if(&*PRIM_TOKENS)?;
        if let Some(input) = input {
            match input.kind {
                TokenKind::Literal(literal) => match literal {
                    Literal::Integer => {
                        let value = input
                            .value
                            .parse::<i64>()
                            .map_err(|err| parser.map_error(err))?;
                        Ok(Some(prim::Prim::Integer(prim::Integer::I64(value))))
                    }
                    Literal::Float => {
                        let value = input
                            .value
                            .parse::<f32>()
                            .map_err(|err| parser.map_error(err))?;
                        Ok(Some(prim::Prim::Float(prim::Float::F32(value))))
                    }
                    Literal::String => {
                        let len = input.value.len();
                        let value =
                            unescape::unescape(&input.value.get(1..len - 1).ok_or_else(|| {
                                parser.map_error(errors::ParserError::QuoteError)
                            })?)
                            .ok_or_else(|| parser.map_error(errors::SyntaxError::UnknownEscape))?;
                        Ok(Some(prim::Prim::String(value)))
                    }
                    Literal::MultiString => {
                        let len = input.value.len();
                        let value =
                            unescape::unescape(&input.value.get(3..len - 3).ok_or_else(|| {
                                parser.map_error(errors::ParserError::QuoteError)
                            })?)
                            .ok_or_else(|| parser.map_error(errors::SyntaxError::UnknownEscape))?;
                        Ok(Some(prim::Prim::String(value)))
                    }
                },
                _ => unreachable!("the tokens should be filtered"),
            }
        } else {
            Ok(None)
        }
    }
}
