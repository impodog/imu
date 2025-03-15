use crate::prelude::*;
use ast::expr::Body;

pub struct BodyConv;

impl Converter for BodyConv {
    type Input = Body;
}

impl Convert<Value> for BodyConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        ctx.body_mut().push_locals();
        for bind in input.bind.iter() {
            convs::BindConv.convert(ctx, bind)?;
        }
        let mut result = None;
        for expr in input.body.iter() {
            result = convs::ExprConv::default().convert(ctx, expr)?;
        }
        let result = if input.unit {
            Value::default()
        } else {
            result.unwrap_or_default()
        };

        let locals = ctx
            .body_mut()
            .pop_locals()
            .expect("context should contain a set of locals after expression body");
        let body = ctx.body_mut();
        for value in locals.value.into_values() {
            body.drop_value(&value)?;
        }
        Ok(result)
    }
}
