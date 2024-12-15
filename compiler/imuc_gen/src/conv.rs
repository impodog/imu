use crate::prelude::*;

pub trait Conv<Output> {
    type Input;

    fn convert(self, ctx: &mut Ctx, input: Self::Input) -> Result<Output>;
}
