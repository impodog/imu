use crate::prelude::*;

pub struct ElemExprRule;

impl Rule for ElemExprRule {
    type Output = expr::Expr;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if let Some(prim) = rules::PrimRule.parse(parser)? {
            Ok(Some(expr::Expr::Prim(prim)))
        } else if let Some(value) = rules::ValueRule.parse(parser)? {
            Ok(Some(expr::Expr::Value(value)))
        } else if let Some(body) = rules::BodyRule.parse(parser)? {
            Ok(Some(expr::Expr::Body(body)))
        } else if let Some(flow) = rules::FlowRule.parse(parser)? {
            Ok(Some(expr::Expr::Flow(flow)))
        } else if let Some(tuple) = rules::TupleExprRule.parse(parser)? {
            Ok(Some(expr::Expr::Tuple(tuple)))
        } else if let Some(struct_stmt) = rules::StructExprRule.parse(parser)? {
            Ok(Some(expr::Expr::Struct(struct_stmt)))
        } else {
            Ok(None)
        }
    }
}
