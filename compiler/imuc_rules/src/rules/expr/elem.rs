use crate::prelude::*;
use imuc_lexer::token::{Keyword, ResTy};

lazy_tokens!(
    CastTyTokens,
    ResTy::I8,
    ResTy::I16,
    ResTy::I32,
    ResTy::I64,
    ResTy::F32,
    ResTy::F64
);

pub struct ElemExprRule;

impl Rule for ElemExprRule {
    type Output = expr::Expr;

    #[allow(clippy::manual_map)]
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();
        let expr = if let Some(prim) = rules::PrimRule.parse(parser)? {
            Some(expr::Expr::Prim(prim))
        } else if let Some(value) = rules::ValueRule.parse(parser)? {
            Some(expr::Expr::Value(value))
        } else if let Some(body) = rules::BodyRule.parse(parser)? {
            Some(expr::Expr::Body(body))
        } else if let Some(flow) = rules::FlowRule.parse(parser)? {
            Some(expr::Expr::Flow(flow))
        } else if let Some(tuple) = rules::TupleExprRule.parse(parser)? {
            Some(tuple)
        } else if let Some(struct_stmt) = rules::CusExprRule.parse(parser)? {
            Some(expr::Expr::Cus(struct_stmt))
        } else if parser.next_if(&TokenKind::Keyword(Keyword::Mit))?.is_some() {
            let mut index = 0;
            while parser.next_if(&TokenKind::Keyword(Keyword::Mit))?.is_some() {
                index += 1;
            }
            let expr = rules::ExprRule { end: () }
                .parse(parser)?
                .unwrap_or(expr::Expr::Prim(prim::Prim::Unit));
            Some(expr::Expr::Mit(expr::Mit {
                index,
                expr: Box::new(expr),
                span: parser.file_info().into_span(cursor_begin),
            }))
        } else {
            None
        };
        // Search for suffix "as" statements
        let expr = match expr {
            Some(expr) => {
                if parser.next_if(&TokenKind::Keyword(Keyword::As))?.is_some() {
                    let ty = parser.next_expected(&CastTyTokens)?;
                    let ty = match ty.kind {
                        TokenKind::ResTy(ty) => ty,
                        _ => filtered!(),
                    };
                    let expr = expr::Expr::Cast(expr::Cast {
                        expr: Box::new(expr),
                        ty,
                        span: parser.file_info().into_span(cursor_begin),
                    });
                    Some(expr)
                } else {
                    Some(expr)
                }
            }
            None => None,
        };
        Ok(expr)
    }
}
