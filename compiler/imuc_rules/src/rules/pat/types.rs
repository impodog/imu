use crate::prelude::*;
use imuc_lexer::token::{Ident, Pair, ResTy, Symbol, UnOp};

pub struct TemplArgsRule;

impl Rule for TemplArgsRule {
    type Output = Vec<pat::Type>;

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

                let item = TypeRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Type".to_owned(),
                        context: "template arguments".to_owned(),
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

pub struct TypeRule;

lazy_tokens!(TypeNameTokens, Ident::Type, Ident::Unused);
lazy_tokens!(
    ResTyTokens,
    ResTy::I8,
    ResTy::I16,
    ResTy::I32,
    ResTy::I64,
    ResTy::F32,
    ResTy::F64,
    ResTy::Ptr,
    ResTy::SelfType,
    ResTy::Str
);

impl Rule for TypeRule {
    type Output = pat::Type;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();

        let flags = if parser.next_if(&TokenKind::UnOp(UnOp::Ref))?.is_some() {
            pat::PatFlags::Shared
        } else if parser.next_if(&TokenKind::UnOp(UnOp::Ptr))?.is_some() {
            pat::PatFlags::Stack
        } else {
            pat::PatFlags::Unique
        };
        let name = parser.next_if(&TypeNameTokens)?;

        if let Some(name) = name {
            let name = match name.kind {
                TokenKind::Ident(ident) => match ident {
                    Ident::Type => name.value,
                    Ident::Unused => {
                        return Ok(Some(pat::Type {
                            flags,
                            kind: pat::TypeKind::Wildcard,
                            span: parser.file_info().into_span(cursor_begin),
                        }))
                    }
                    _ => filtered!(),
                },
                _ => filtered!(),
            };
            let name = parser.look_up.insert(name);

            Ok(Some(pat::Type {
                flags,
                kind: pat::TypeKind::Single(name),
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else if let Some(res) = parser.next_if(&ResTyTokens)? {
            let res = match res.kind {
                TokenKind::ResTy(res) => res,
                _ => filtered!(),
            };
            Ok(Some(pat::Type {
                flags,
                kind: pat::TypeKind::Res(res),
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else if parser.next_if(&TokenKind::Pair(Pair::LeftParen))?.is_some() {
            let mut list = Vec::new();
            let mut comma = false;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightParen))?
                    .is_some()
                {
                    break;
                } else if !list.is_empty() && !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                let pat = rules::TypeRule.parse(parser)?.ok_or_else(|| {
                    parser.map_err(errors::SyntaxError::ExpectedIn {
                        expect: "Ty pat".to_owned(),
                        context: "tuple type".to_owned(),
                    })
                })?;

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                list.push(pat);
            }
            Ok(Some(pat::Type {
                flags,
                kind: pat::TypeKind::Tuple(list),
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else if flags != pat::PatFlags::Unique {
            Err(parser.map_err(errors::SyntaxError::ExpectedAfter {
                expect: "Type".to_owned(),
                after: TokenKind::UnOp(UnOp::Ref),
            }))
        } else {
            Ok(None)
        }
    }
}
