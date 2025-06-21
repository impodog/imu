use crate::prelude::*;
use imuc_ast::item::For;

pub struct ForConv;

impl Converter for ForConv {
    type Input = For;
}

impl Convert<()> for ForConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let ty = convs::TypeConv.convert(ctx, &input.ty)?.ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.ty.span())
                    .with_head("Type should not contain wildcard in type implementation"),
            );
            SendError::new_error()
        })?;
        ctx.body_mut().push_self_ty(ty.clone());
        for item in input.items.iter() {
            convs::ItemConv.convert(ctx, item).inspect_err(|_| {
                ctx.push_error(
                    ConvError::new(Severity::Note, input.ty.span())
                        .with_text("In for block", format!("In for block of type {}", ty.name)),
                );
            })?;
        }
        ctx.body_mut()
            .pop_self_ty()
            .expect("For block should have a self_ty after body conversion");
        Ok(())
    }
}
