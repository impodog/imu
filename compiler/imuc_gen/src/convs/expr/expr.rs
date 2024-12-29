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
            _ => {
                todo!("expression conversion")
            }
        }
    }
}
