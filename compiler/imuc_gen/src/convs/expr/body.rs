use crate::prelude::*;
use ast::expr::Body;

pub struct BodyConv;

impl Converter for BodyConv {
    type Input = Body;
}

pub fn handle_body_errors(ctx: &mut Ctx, prev_error_len: usize) {
    todo!()
}

impl Convert<Value> for BodyConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        ctx.body_mut().push_locals();
        ctx.body_mut().push_stack_record();
        for bind in input.bind.iter() {
            convs::BindConv.convert(ctx, bind)?;
        }
        let mut result = None;
        let prev_error_len = ctx.error_queue.read().unwrap().len();
        for expr in input.body.iter() {
            match convs::ExprConv::default().convert(ctx, expr) {
                Ok(value) => {
                    result = value;
                }
                Err(_) => {
                    handle_body_errors(ctx, prev_error_len);
                }
            }
        }
        let result = if input.unit {
            Value::default()
        } else {
            result.unwrap_or_default()
        };

        let _locals = ctx
            .body_mut()
            .pop_locals()
            .expect("context should contain a set of locals after expression body");
        // Adjust return value pointer
        debug_assert!(ctx.body_mut().pop_stack_record());
        let size = result.ty.size_or(input.span())?;
        // NOTE: This may be a overlapping duplicate
        ctx.body_mut().push(Cmd::Dupli(size, result.ptr));
        let ptr = ctx.body_mut().push_stack(size);
        Ok(Value { ty: result.ty, ptr })
    }
}
