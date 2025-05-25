use crate::prelude::*;
use ast::module::Module;

pub struct SubmoduleConv;

impl Converter for SubmoduleConv {
    type Input = Module;
}

impl Convert<()> for SubmoduleConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        for sub in input.sub_nodes.iter() {
            ctx.push_body(sub.name.as_str(), None, true);
            SubmoduleConv.convert(ctx, &sub.module)?;
            ctx.pop_body();
        }
        for item in input.items.iter() {
            convs::ItemConv.convert(ctx, item)?;
        }
        Ok(())
    }
}
