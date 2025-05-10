use crate::prelude::*;
use imuc_lexer::token::{Keyword, Pair};

pub struct IfRule;

impl Rule for IfRule {
    type Output = flow::If;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        if parser.next_if(&TokenKind::Keyword(Keyword::If))?.is_some() {
            let cond = rules::ExprRule {
                end: TokenKind::Pair(Pair::LeftBrace),
            }
            .parse(parser)?
            .ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Expr".to_owned(),
                    after: TokenKind::Keyword(Keyword::If),
                })
            })?;
            let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedIn {
                    expect: "Body".to_owned(),
                    context: "if statement".to_owned(),
                })
            })?;
            Ok(Some(flow::If {
                cond: Box::new(cond),
                body,
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else {
            Ok(None)
        }
    }
}

pub struct IfElseRule;

impl Rule for IfElseRule {
    type Output = flow::IfElse;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        let mut ifs = Vec::new();
        let end = loop {
            if let Some(if_stmt) = IfRule.parse(parser)? {
                ifs.push(if_stmt);
            } else if !ifs.is_empty() {
                return Err(parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "if statement".to_owned(),
                    after: TokenKind::Keyword(Keyword::Else),
                }));
            } else {
                break None;
            };
            if parser
                .next_if(&TokenKind::Keyword(Keyword::Else))?
                .is_some()
            {
                if parser
                    .peek()?
                    .is_some_and(|token| token.kind == TokenKind::Keyword(Keyword::If))
                {
                    // Continue parsing remaining ifs
                } else {
                    let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Body".to_owned(),
                            context: "if statement".to_owned(),
                        })
                    })?;
                    break Some(body);
                }
            } else {
                break None;
            }
        };
        if let Some(ifs) = nonempty::NonEmpty::from_vec(ifs) {
            Ok(Some(flow::IfElse {
                ifs,
                end,
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else {
            Ok(None)
        }
    }
}
