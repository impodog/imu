use crate::prelude::*;
use std::io::Write;

/// Trait for input/output of a IR object
pub trait Rw: Sized {
    /// Reads the IR from the input, returning any error
    fn read(input: impl IrRead) -> Result<Self>;

    /// Writing the IR to the output, returning any error
    fn write(&self, output: impl Write) -> Result<()>;
}
