use crate::prelude::*;
use imuc_lexer::token::Keyword;

pub struct LoopRule;

impl Rule for LoopRule {
    type Output = flow::Loop;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        if parser
            .next_if(&TokenKind::Keyword(Keyword::Loop))?
            .is_some()
        {
            let body = rules::BodyRule.parse(parser)?.ok_or_else(|| {
                parser.map_err(errors::SyntaxError::ExpectedAfter {
                    expect: "Body".to_owned(),
                    after: TokenKind::Keyword(Keyword::Loop),
                })
            })?;
            Ok(Some(flow::Loop {
                body,
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else {
            Ok(None)
        }
    }
}
