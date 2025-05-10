use crate::prelude::*;
use imuc_ast::item::For;

pub struct ForConv;

impl Converter for ForConv {
    type Input = For;
}

impl Convert<()> for ForConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let ty = convs::PatConv {
            name: None,
            requires_ty: true,
            discard_name_warn: true,
        }
        .convert(ctx, &input.ty)?
        .expect("PatConv should not return None when requires_ty is true");
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
