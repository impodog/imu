use crate::prelude::*;
use ast::{item::Cus, module::Public};

pub struct CusConv {
    /// The complete unique name of the cus type
    pub name: StrRef,
    /// An alias of the type implied in current module
    pub alias: StrRef,
    pub public: Public,
}

impl Converter for CusConv {
    type Input = Cus;
}

impl Convert<()> for CusConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let CusConv {
            name,
            alias,
            public,
        } = self;
        let ty = convs::PatConv {
            requires_ty: true,
            discard_name_warn: true,
            name: Some(name.clone()),
            public,
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

        // Add to import aliases
        if !ctx.body_mut().insert_import(alias, name) {
            ctx.push_error(
                ConvError::new(Severity::Warn, input.elem.span())
                    .with_head("Cus with the same name, this one will not have an alias"),
            );
        }
        Ok(())
    }
}
