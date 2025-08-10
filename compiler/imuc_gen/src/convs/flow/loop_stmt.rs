use crate::prelude::*;
use ast::flow::Loop;
use convs::ExprSolver;

/// Loop conversion structure:
/// Jump to index + 2 (begin the loop)
/// Jump to the end of the loop (the loop pointer stored in loop record)
/// Start of the loop
/// ... (loop body)
/// Jump to the start of the loop
/// End of the loop
pub struct LoopConv {
    pub solver: ExprSolver,
}

impl Converter for LoopConv {
    type Input = Loop;
}

impl Convert<Value> for LoopConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let body = ctx.body_mut();

        let loop_begin_index = body.len() + 2;
        body.push_void(Cmd::Jump(Ptr::new(loop_begin_index)));

        let loop_record_index = body.len();
        // Force align stack for mit value
        body.push_loop_record();
        // Placeholder for a pointer out of the loop
        body.push_void(Cmd::End);

        let value = convs::BodyConv::default()
            .convert(ctx, &input.body)
            .inspect_err(|_err| {
                debug_assert!(ctx.body_mut().pop_stack_record(Bytes::null()));
                debug_assert!(ctx.body_mut().pop_loop_record().is_some());
            })?;
        if !value.ty.test_eq(&Ty::unit()) {
            ctx.push_error(
                ConvError::new(Severity::Warn, input.body.span)
                    .with_head("Return values of a loop statement will be ignored"),
            );
        }

        let push_error_fn = ctx.push_error_fn();

        let body = ctx.body_mut();
        // Add loop jump-back
        body.push_void(Cmd::Jump(Ptr::new(loop_begin_index)));

        // Replace loop record jump out command
        let end_index = body.len();
        body.replace(loop_record_index, Cmd::Jump(Ptr::new(end_index)));

        let loop_record = body
            .pop_loop_record()
            .expect("A loop record should exist after pushing");
        let ty = loop_record
            .ty
            .get()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                push_error_fn(
                    ConvError::new(Severity::Warn, input.span())
                        .with_head("This loop never ends, use \"mit\" to quit"),
                );
                Ty::unit()
            });
        let ty_size = ty.size_or(input.span())?;
        body.force_stack_to(loop_record.stack + ty_size);
        // Calculate the return value pointer
        let ptr = body.stack() - ty_size;
        Ok(Value { ty, ptr })
    }
}
