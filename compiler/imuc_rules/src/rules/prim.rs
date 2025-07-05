use crate::prelude::*;
use imuc_lexer::token::Literal;

lazy_tokens!(
    PrimTokens,
    Literal::Integer,
    Literal::Float,
    Literal::I8,
    Literal::I16,
    Literal::I32,
    Literal::I64,
    Literal::F32,
    Literal::F64,
    Literal::MultiString,
    Literal::String
);

pub struct PrimRule;

impl PrimRule {
    fn parse_int<T>(value: &str, build: fn(T) -> prim::Integer) -> Result<prim::Integer>
    where
        T: lexical::FromLexicalWithOptions<Options = lexical::ParseIntegerOptions>
            + lexical::FromLexical,
    {
        let value = match value.rfind('I') {
            Some(index) => &value[..index],
            None => value,
        };
        let mut iter = value.chars();
        let first = iter.next();
        let second = iter.next();
        let options = lexical::ParseIntegerOptions::new();
        let value = match (first, second) {
            (Some('0'), Some('x' | 'X')) => {
                const FORMAT: u128 = lexical::NumberFormatBuilder::new().radix(16).build();
                let value: T = lexical::parse_with_options::<_, _, FORMAT>(&value[2..], &options)?;
                build(value)
            }
            (Some('0'), Some('b' | 'B')) => {
                const FORMAT: u128 = lexical::NumberFormatBuilder::new().radix(2).build();
                let value: T = lexical::parse_with_options::<_, _, FORMAT>(&value[2..], &options)?;
                build(value)
            }
            _ => {
                let value: T = lexical::parse(value)?;
                build(value)
            }
        };
        Ok(value)
    }

    fn parse_float<T>(value: &str, build: fn(T) -> prim::Float) -> Result<prim::Float>
    where
        T: lexical::FromLexicalWithOptions<Options = lexical::ParseFloatOptions>
            + lexical::FromLexical,
    {
        let value: T = lexical::parse(value.as_bytes())?;
        Ok(build(value))
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
                    TokenKind::Literal(literal) => {
                        match literal {
                            Literal::Integer => Self::parse_int(input.value, prim::Integer::Any)
                                .map(prim::Prim::Integer),
                            Literal::Float => Self::parse_float(input.value, prim::Float::Any)
                                .map(prim::Prim::Float),
                            Literal::I8 => Self::parse_int(input.value, prim::Integer::I8)
                                .map(prim::Prim::Integer),
                            Literal::I16 => Self::parse_int(input.value, prim::Integer::I16)
                                .map(prim::Prim::Integer),
                            Literal::I32 => Self::parse_int(input.value, prim::Integer::I32)
                                .map(prim::Prim::Integer),
                            Literal::I64 => Self::parse_int(input.value, prim::Integer::I64)
                                .map(prim::Prim::Integer),
                            Literal::F32 => Self::parse_float(input.value, prim::Float::F32)
                                .map(prim::Prim::Float),
                            Literal::F64 => Self::parse_float(input.value, prim::Float::F64)
                                .map(prim::Prim::Float),
                            Literal::String => {
                                let len = input.value.len();
                                let value =
                                    unescape::unescape(input.value.get(1..len - 1).ok_or_else(
                                        || parser.map_err(errors::ParserError::QuoteError),
                                    )?)
                                    .ok_or_else(|| errors::SyntaxError::UnknownEscape.into());
                                value.map(prim::Prim::String)
                            }
                            Literal::MultiString => {
                                let len = input.value.len();
                                let value =
                                    unescape::unescape(input.value.get(3..len - 3).ok_or_else(
                                        || parser.map_err(errors::ParserError::QuoteError),
                                    )?)
                                    .ok_or_else(|| errors::SyntaxError::UnknownEscape.into());
                                value.map(prim::Prim::String)
                            }
                        }
                    }
                    _ => unreachable!("the tokens should be filtered"),
                };
            prim.map_err(|err| parser.map_err(err)).map(Some)
        } else {
            Ok(None)
        }
    }
}
