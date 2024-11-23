use crate::prelude::*;
use imuc_lexer::token::{BinOp, Ident, Pair, Symbol};

pub struct TemplDefRule;

impl Rule for TemplDefRule {
    type Output = Vec<item::Templ>;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&TokenKind::Pair(Pair::LeftBracket))?;
        let mut comma = true;
        if input.is_some() {
            let mut templ = Vec::new();
            loop {
                let next = parser.next_if(&TokenKind::Pair(Pair::RightBracket))?;
                if next.is_some() {
                    break;
                } else if comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                let item = TemplItemRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Type".to_owned(),
                        context: "template definition".to_owned(),
                    })
                })?;

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                templ.push(item);
            }
            Ok(Some(templ))
        } else {
            Ok(None)
        }
    }
}

lazy_tokens!(TemplItemTokens, Ident::Type, Ident::Unused);

pub struct TemplItemRule;

impl Rule for TemplItemRule {
    type Output = item::Templ;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&TemplItemTokens)?;
        if let Some(input) = input {
            let name = match input.kind {
                TokenKind::Ident(ident) => match ident {
                    Ident::Type => parser.look_up.insert(input.value),
                    Ident::Unused => return Ok(Some(item::Templ::Unused)),
                    _ => filtered!(),
                },
                _ => filtered!(),
            };
            let colon = parser.next_if(&TokenKind::Symbol(Symbol::Colon))?;
            let mut req = Vec::new();
            if colon.is_some() {
                // When requirements are present for the type
                loop {
                    let ty = rules::TypeRule.parse(parser)?.ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Type".to_owned(),
                            context: "template requirements".to_owned(),
                        })
                    })?;
                    req.push(ty);
                    if parser.next_if(&TokenKind::BinOp(BinOp::Add))?.is_none() {
                        break;
                    }
                }
                Ok(Some(item::Templ::Item(item::TemplItem { name, req })))
            } else {
                // Or, use the default requirements
                Ok(Some(item::Templ::Item(item::TemplItem {
                    name,
                    req: Vec::default(),
                })))
            }
        } else {
            Ok(None)
        }
    }
}