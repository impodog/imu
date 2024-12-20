use crate::prelude::*;

/// Defines the input AST type of the converter
pub trait Converter {
    type Input;
}

/// Defines a specific conversion option of the converter
pub trait Convert<Output>
where
    Self: Converter,
{
    fn convert(self, ctx: &mut Ctx, input: Self::Input) -> Result<Output>;
}
