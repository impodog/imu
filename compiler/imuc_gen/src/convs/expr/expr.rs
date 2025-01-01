use crate::prelude::*;
use ast::expr::Expr;
use ctx::Value;

pub struct ExprConv;

impl Converter for ExprConv {
    type Input = Expr;
}

impl Convert<Option<Value>> for ExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        match input {
            Expr::Prim(prim) => {
                let value = convs::PrimConv.convert(ctx, prim)?;
                Ok(Some(value))
            }
            Expr::Value(value) => convs::ValueConv.convert(ctx, value),
            Expr::UnExpr(un_expr) => convs::UnExprConv.convert(ctx, un_expr),
            Expr::BinExpr(bin_expr) => convs::BinExprConv.convert(ctx, bin_expr),
            _ => {
                todo!("expression conversion")
            }
        }
    }
}
