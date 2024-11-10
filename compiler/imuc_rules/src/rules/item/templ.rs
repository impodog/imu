use crate::prelude::*;
use imuc_lexer::token::{BinOp, Ident, Pair, Symbol};

pub struct TemplRule;

impl Rule for TemplRule {
    type Output = Vec<item::Templ>;

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&TokenKind::Pair(Pair::LeftParen))?;
        if input.is_some() {
            let mut templ = Vec::new();
            loop {
                let next = parser.next_if(&TokenKind::Pair(Pair::RightParen))?;
                if next.is_some() {
                    break;
                } else {
                    let item = TemplItemRule.parse(parser)?.ok_or_else(|| {
                        parser.map_error(errors::SyntaxError::ExpectedIn {
                            expect: "Type".to_owned(),
                            context: "template definition".to_owned(),
                        })
                    })?;
                    templ.push(item);
                }
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

    fn parse<'s, I>(&self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let input = parser.next_if(&TemplItemTokens)?;
        if let Some(input) = input {
            let name = match input.kind {
                TokenKind::Ident(ident) => match ident {
                    Ident::Type => input.value.to_name(),
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
                        parser.map_error(errors::SyntaxError::ExpectedIn {
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
