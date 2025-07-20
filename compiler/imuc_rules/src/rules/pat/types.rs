use crate::prelude::*;
use imuc_lexer::token::{Ident, Keyword, Pair, ResTy, Symbol, UnOp};
pub struct TypeRule;

lazy_tokens!(TypeNameTokens, Ident::Type, Ident::Unused);
lazy_tokens!(
    ResTyTokens,
    ResTy::Unit,
    ResTy::Bool,
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

        let has_prefix = rules::PrefixRule.parse(parser)?.is_some();
        // Raises an error if has_prefix is true
        let should_not_have_prefix = |parser: &mut Parser<'s, I>| -> Result<()> {
            if has_prefix {
                Err(parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "type name".to_owned(),
                    after: imuc_lexer::TokenKind::Prefix,
                }))
            } else {
                Ok(())
            }
        };

        if let Some(name) = parser.next_if(&TypeNameTokens)? {
            let name = match name.kind {
                TokenKind::Ident(ident) => match ident {
                    Ident::Type => name.value,
                    Ident::Unused => {
                        should_not_have_prefix(parser)?;

                        return Ok(Some(pat::Type {
                            flags,
                            kind: pat::TypeKind::Wildcard,
                            span: parser.file_info().into_span(cursor_begin),
                        }));
                    }
                    _ => filtered!(),
                },
                _ => filtered!(),
            };
            let name = parser.look_up.insert(name);
            if has_prefix {
                let prefix = parser
                    .pop_prefix()
                    .expect("should have a prefix after parsing one");
                Ok(Some(pat::Type {
                    flags,
                    kind: pat::TypeKind::Prefixed(name::PrefixedName::new(prefix, name)),
                    span: parser.file_info().into_span(cursor_begin),
                }))
            } else {
                Ok(Some(pat::Type {
                    flags,
                    kind: pat::TypeKind::Single(name),
                    span: parser.file_info().into_span(cursor_begin),
                }))
            }
        } else if parser
            .next_if(&TokenKind::Keyword(Keyword::Decl))?
            .is_some()
        {
            parser.next_expected(&TokenKind::Pair(Pair::LeftParen))?;
            let expr = rules::ExprRule { end: () }.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "expression".to_owned(),
                    after: TokenKind::Keyword(Keyword::Decl),
                })
            })?;
            parser.next_expected(&TokenKind::Pair(Pair::RightParen))?;
            Ok(Some(pat::Type {
                flags,
                kind: pat::TypeKind::Decl(Box::new(expr)),
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else if let Some(res) = parser.next_if(&ResTyTokens)? {
            should_not_have_prefix(parser)?;

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
            should_not_have_prefix(parser)?;

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
            // Return Unit type if it is an empty pair of parentheses
            if list.is_empty() {
                Ok(Some(pat::Type {
                    flags,
                    kind: pat::TypeKind::Res(ResTy::Unit),
                    span: parser.file_info().into_span(cursor_begin),
                }))
            } else if list.len() == 1 && !comma {
                let ty = list
                    .pop()
                    .expect("list should contain a value since its len is 1");
                Ok(Some(ty))
            } else {
                Ok(Some(pat::Type {
                    flags,
                    kind: pat::TypeKind::Tuple(list),
                    span: parser.file_info().into_span(cursor_begin),
                }))
            }
        } else if flags != pat::PatFlags::Unique {
            Err(parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Type".to_owned(),
                context: "Ref/Ptr".to_owned(),
            }))
        } else {
            Ok(None)
        }
    }
}
