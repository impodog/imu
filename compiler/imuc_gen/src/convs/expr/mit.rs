use crate::prelude::*;
use ast::expr::Mit;

pub struct MitConv;

impl Converter for MitConv {
    type Input = Mit;
}

impl Convert<()> for MitConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let hint = ctx
            .body()
            .loop_record(input.index)
            .and_then(|loop_record| loop_record.ty.get().cloned());
        let value = convs::ExprConv::default()
            .with_hint(hint)
            .convert(ctx, input.expr.as_ref())?
            .unwrap_or_else(Default::default);
        let loop_record = ctx.body().loop_record(input.index).ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span())
                    .with_head("Attempt to escape loops more than there is"),
            );
            SendError::new_error()
        })?;
        if !loop_record.check_ty(&value.ty) {
            ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                "Mit statement to the same loop types mismatch",
                format!(
                        "Expected {}, found {}",
                        loop_record
                            .ty
                            .get()
                            .expect("Loop record should have a type when check_ty returns false")
                            .name,
                        value.ty.name
                    ),
            ));
            return Err(SendError::new_error());
        }
        let stack = loop_record.stack;
        let ptr = loop_record.ptr;
        let ty_size = value.ty.size_or(input.span())?;
        let body = ctx.body_mut();
        // Copy the return value, while stack change will be done by the loop
        body.push_void(Cmd::Overwrite(ty_size, value.ptr, stack));
        // Jump to loop quit location
        body.push_void(Cmd::Jump(ptr));
        Ok(())
    }
}
