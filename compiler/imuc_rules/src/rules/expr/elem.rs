use crate::prelude::*;

pub struct ElemExprRule;

impl Rule for ElemExprRule {
    type Output = expr::Expr;

    #[allow(clippy::manual_map)]
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
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
        } else if let Some(struct_stmt) = rules::StructExprRule.parse(parser)? {
            Some(expr::Expr::Cus(struct_stmt))
        } else {
            None
        };
        Ok(expr)
    }
}
