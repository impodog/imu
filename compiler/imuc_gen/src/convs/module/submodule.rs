use crate::prelude::*;
use ast::module::Module;

pub struct SubmoduleConv;

impl Converter for SubmoduleConv {
    type Input = Module;
}

impl Convert<()> for SubmoduleConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        for sub in input.sub_nodes.iter() {
            // Compile submodule
            ctx.push_body(sub.name.as_str(), false, None, true);
            SubmoduleConv.convert(ctx, &sub.module)?;
            let inner_body = ctx
                .pop_body()
                .expect("Should contain a body after compiling");
            // Add the alias to the submodule locally to the current module
            ctx.body_mut()
                .insert_import(sub.name.clone(), inner_body.name().into());
        }
        for item in input.items.iter() {
            convs::ItemConv.convert(ctx, item)?;
        }
        Ok(())
    }
}
