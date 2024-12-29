use crate::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};

pub const PTR_SIZE: u32 = std::mem::size_of::<u32>() as u32;

/// Represent the number of bytes, or a pointer to the stack
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(u32);
pub type Ptr = Bytes;

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

impl Bytes {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// Creates a representation of bytes with length equal to [`u32`]
    pub const fn ptr() -> Self {
        Self(PTR_SIZE)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum NumBytes {
    I8,
    I16,
    I32,
    I64,
}

impl TryFrom<char> for NumBytes {
    type Error = Error;
    fn try_from(value: char) -> std::result::Result<Self, Self::Error> {
        use NumBytes::*;
        let value = match value {
            'b' => I8,
            'd' => I16,
            'q' => I32,
            'o' => I64,
            _ => return Err(errors::IrError::NoSuchCommandMod(value.to_string()).into()),
        };
        Ok(value)
    }
}

impl TryFrom<&str> for NumBytes {
    type Error = Error;
    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        if let Some(ch) = value.chars().next() {
            if value.len() == 1 {
                return ch.try_into();
            }
        }
        Err(errors::IrError::NoSuchCommandMod(value.to_owned()).into())
    }
}

impl From<NumBytes> for char {
    fn from(value: NumBytes) -> Self {
        use NumBytes::*;
        match value {
            I8 => 'b',
            I16 => 'd',
            I32 => 'q',
            I64 => 'o',
        }
    }
}
