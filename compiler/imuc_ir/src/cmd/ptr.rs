use crate::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Represent the number of bytes, or a pointer to the stack
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(u32);
pub type Ptr = Bytes;

/// New type wrapper around [`Ptr`] indicating a creation of new pointers, instead of querying
#[derive(Default, Clone, Copy, Debug)]
pub struct Alloc(pub Ptr);

impl Add<Bytes> for Bytes {
    type Output = Bytes;
    fn add(self, rhs: Bytes) -> Self::Output {
        Bytes(self.0.saturating_add(rhs.0))
    }
}

impl AddAssign for Bytes {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}

impl Sub<Bytes> for Bytes {
    type Output = Bytes;
    fn sub(self, rhs: Bytes) -> Self::Output {
        Bytes(self.0.saturating_sub(rhs.0))
    }
}

impl SubAssign for Bytes {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
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
