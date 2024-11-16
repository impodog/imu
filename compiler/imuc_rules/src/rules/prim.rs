use crate::prelude::*;
use imuc_lexer::token::Literal;

lazy_tokens!(
    PrimTokens,
    Literal::Integer,
    Literal::Float,
    Literal::MultiString,
    Literal::String
);

pub struct PrimRule;

impl PrimRule {
    fn parse_int(value: &str) -> Result<prim::Integer> {
        let mut iter = value.chars();
        let first = iter.next();
        let second = iter.next();
        let options = lexical::ParseIntegerOptions::new();
        let value = match (first, second) {
            (Some('0'), Some('x')) => {
                const FORMAT: u128 = lexical::NumberFormatBuilder::new().radix(16).build();
                let value: i64 =
                    lexical::parse_with_options::<_, _, FORMAT>(&value[2..], &options)?;
                prim::Integer::I64(value)
            }
            (Some('0'), Some('b')) => {
                const FORMAT: u128 = lexical::NumberFormatBuilder::new().radix(2).build();
                let value: i64 =
                    lexical::parse_with_options::<_, _, FORMAT>(&value[2..], &options)?;
                prim::Integer::I64(value)
            }
            _ => {
                let value: i64 = lexical::parse(value)?;
                prim::Integer::I64(value)
            }
        };
        Ok(value)
    }

    fn parse_float(value: &str) -> Result<prim::Float> {
        let value: f32 = lexical::parse(value.as_bytes())?;
        Ok(prim::Float::F32(value))
    }
}

impl Rule for PrimRule {
    type Output = prim::Prim;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&PrimTokens)?;
        if let Some(input) = input {
            let prim =
                match input.kind {
                    TokenKind::Literal(literal) => match literal {
                        Literal::Integer => Self::parse_int(input.value).map(prim::Prim::Integer),
                        Literal::Float => Self::parse_float(input.value).map(prim::Prim::Float),
                        Literal::String => {
                            let len = input.value.len();
                            let value =
                                unescape::unescape(&input.value.get(1..len - 1).ok_or_else(
                                    || parser.map_err(errors::ParserError::QuoteError),
                                )?)
                                .ok_or_else(|| errors::SyntaxError::UnknownEscape.into());
                            value.map(prim::Prim::String)
                        }
                        Literal::MultiString => {
                            let len = input.value.len();
                            let value =
                                unescape::unescape(&input.value.get(3..len - 3).ok_or_else(
                                    || parser.map_err(errors::ParserError::QuoteError),
                                )?)
                                .ok_or_else(|| errors::SyntaxError::UnknownEscape.into());
                            value.map(prim::Prim::String)
                        }
                    },
                    _ => unreachable!("the tokens should be filtered"),
                };
            prim.map_err(|err| parser.map_err(err)).map(Some)
        } else {
            Ok(None)
        }
    }
}
