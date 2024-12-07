use crate::prelude::*;
use std::ops::Add;

/// Represent the number of bytes, or a pointer to the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(u32);
pub type Ptr = Bytes;

impl Add<Bytes> for Bytes {
    type Output = Bytes;
    fn add(self, rhs: Bytes) -> Self::Output {
        Bytes(self.0.saturating_add(rhs.0))
    }
}

impl From<Bytes> for usize {
    fn from(value: Bytes) -> Self {
        value.0 as usize
    }
}

impl From<Bytes> for u32 {
    fn from(value: Bytes) -> Self {
        value.0
    }
}

impl Rw for Bytes {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let value = input.read_until(' ')?;
        let value = value.parse::<u32>()?;
        Ok(Self(value))
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        write!(output, "{}", self.0)?;
        Ok(())
    }
}
