use crate::prelude::*;
use ast::expr::Body;

#[derive(Default)]
pub struct BodyConv {
    /// Determines if the stack record has been previously set. This is useful when you want to add
    /// some manual commands before/after the body, such as loading function arguments,
    /// or loop mit handling
    pub pre_stack_record: bool,
}

impl Converter for BodyConv {
    type Input = Body;
}

impl Convert<Value> for BodyConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        // Push internal body
        ctx.body_mut().push_locals();
        if !self.pre_stack_record {
            ctx.body_mut().push_stack_record();
        }
        for bind in input.bind.iter() {
            convs::BindConv.convert(ctx, bind)?;
        }
        let mut result = None;
        for expr in input.body.iter() {
            match convs::ExprConv::default().convert(ctx, expr) {
                Ok(value) => {
                    result = value;
                }
                Err(_) => {
                    if ctx.error_queue.read().unwrap().len() >= crate::conv::ERROR_QUEUE_LIMIT {
                        return Err(SendError::new_error());
                    }
                    result = None;
                }
            }
        }
        let result = if input.unit {
            Value::default()
        } else {
            result.unwrap_or_default()
        };

        // Remove the internal body
        let _locals = ctx
            .body_mut()
            .pop_locals()
            .expect("context should contain a set of locals after expression body");

        // Adjust return value pointer
        let size = result.ty.size_or(input.span())?;
        let stack_record = ctx
            .body_mut()
            .stack_record()
            .expect("a stack record should exist after body conversion");
        ctx.body_mut()
            .push(Cmd::Overwrite(size, stack_record, result.ptr));

        debug_assert!(ctx.body_mut().pop_stack_record(size));

        let ptr = ctx.body_mut().stack() - size;
        Ok(Value { ty: result.ty, ptr })
    }
}
