use crate::prelude::*;
use ast::expr::Body;
use ctx::Value;

pub struct BodyConv;

impl Converter for BodyConv {
    type Input = Body;
}

impl Convert<Value> for BodyConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        ctx.push_body();
        for bind in input.bind.iter() {
            convs::BindConv.convert(ctx, bind)?;
        }
        let mut result = None;
        for expr in input.body.iter() {
            result = convs::ExprConv.convert(ctx, expr)?;
        }
        let result = if input.unit {
            Value::default()
        } else {
            result.unwrap_or_default()
        };
        ctx.pop_body();
        Ok(result)
    }
}
