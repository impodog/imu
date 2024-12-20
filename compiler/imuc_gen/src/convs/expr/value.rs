use crate::prelude::*;

pub struct ValueConv;

impl Converter for ValueConv {
    type Input = ast::expr::Value;
}

impl Convert<Ptr> for ValueConv {
    fn convert(self, ctx: &mut Ctx, input: Self::Input) -> Result<Ptr> {
        todo!()
    }
}
