use crate::prelude::*;
use ast::bind::Bind;

pub struct BindConv;

impl Converter for BindConv {
    type Input = Bind;
}

impl Convert<()> for BindConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        todo!()
    }
}
