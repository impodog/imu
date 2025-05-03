use crate::prelude::*;
use ast::{item::Cus, module::Public};

pub struct CusConv {
    pub name: StrRef,
    pub public: Public,
}

impl Converter for CusConv {
    type Input = Cus;
}

impl Convert<()> for CusConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let CusConv { name, public } = self;
        let ty = convs::PatConv {
            requires_ty: true,
            discard_name_warn: true,
            name: Some(name.clone()),
        }
        .convert(ctx, &input.elem)?
        .expect("PatConv should not return None when requires_ty is enabled");
        if ty.name != name {
            ctx.push_error(
                ConvError::new(Severity::Error, input.elem.span())
                    .with_head("Cus item requires a compound type"),
            );
        }
        ctx.ty.insert(ty);
        Ok(())
    }
}
