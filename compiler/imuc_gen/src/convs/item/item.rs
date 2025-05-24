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
                let name = ctx.mangle_with_self(input.name.as_str());
                let conv = convs::FunConv {
                    name: name.into(),
                    public: input.public,
                    self_ty: ctx.body().self_ty().cloned(),
                };
                conv.convert(ctx, fun)?;
                Ok(())
            }
            ItemKind::Cus(cus) => {
                let name = ctx.mangle_with_self(input.name.as_str());
                let conv = convs::CusConv {
                    name: name.into(),
                    public: input.public,
                };
                conv.convert(ctx, cus)?;
                Ok(())
            }
            ItemKind::For(for_block) => {
                let conv = convs::ForConv;
                conv.convert(ctx, for_block)?;
                Ok(())
            }
            ItemKind::Val(_val) => {
                todo!("global value conversion")
            }
            ItemKind::Use(import) => {
                let conv = convs::UseConv;
                conv.convert(ctx, import)?;
                Ok(())
            }
        }
    }
}
