use crate::prelude::*;
use ast::item::{Item, ItemKind};

pub struct ItemConv;

impl Converter for ItemConv {
    type Input = Item;
}

impl Convert<()> for ItemConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        match &input.kind {
            ItemKind::Fun(fun) => {
                let name = ctx.mangle(input.name.as_str());
                let name = if let Some(self_ty) = ctx.body().self_ty() {
                    ctx::mangle::mangle_ty_fun(self_ty.name.as_str(), name.as_str())
                } else {
                    name
                };
                let conv = convs::FunConv {
                    name: name.into(),
                    public: input.public,
                    self_ty: ctx.body().self_ty().cloned(),
                };
                conv.convert(ctx, fun)?;
                Ok(())
            }
            _ => {
                todo!("item conversions")
            }
        }
    }
}
