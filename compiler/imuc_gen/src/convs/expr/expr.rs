use crate::prelude::*;
use ast::expr::Expr;

#[derive(Default)]
pub struct ExprConv {
    pub hint: Option<Ty>,
}

impl Converter for ExprConv {
    type Input = Expr;
}

impl Convert<Option<Value>> for ExprConv {
    /// Converts an expression to a value; [`None`] is only returned if the expression is just an
    /// unused name
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        match input {
            Expr::Prim(prim) => convs::PrimConv.convert(ctx, prim).map(Some),
            Expr::Value(value) => convs::ValueConv.convert(ctx, value),
            Expr::UnExpr(un_expr) => convs::UnExprConv.convert(ctx, un_expr).map(Some),
            Expr::BinExpr(bin_expr) => convs::BinExprConv.convert(ctx, bin_expr).map(Some),
            Expr::Body(body) => convs::BodyConv.convert(ctx, body).map(Some),
            Expr::Tuple(tuple) => convs::TupleConv.convert(ctx, tuple).map(Some),
            Expr::Cus(cus) => convs::CusConv.convert(ctx, cus).map(Some),
            _ => {
                todo!("expression conversion")
            }
        }
    }
}
