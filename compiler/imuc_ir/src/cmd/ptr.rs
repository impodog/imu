use crate::prelude::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Returns the representation of the number as [`NumBytes`], if available
const fn number_to_bytes(value: u32) -> Option<NumBytes> {
    match value {
        1 => Some(NumBytes::I8),
        2 => Some(NumBytes::I16),
        4 => Some(NumBytes::I32),
        8 => Some(NumBytes::I64),
        _ => None,
    }
}

pub const PTR_SIZE: u32 = std::mem::size_of::<Ptr>() as u32;
pub const PTR_BYTES: NumBytes = number_to_bytes(PTR_SIZE).unwrap();

/// Represent the number of bytes, or a pointer to the local function stack
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

    /// Creates a representation of bytes from a usize, panics if it exceeds [`u32::MAX`]
    pub fn new_usize(value: usize) -> Self {
        Self(value.try_into().expect("usize should not exceed bounds"))
    }

    /// Creates a representation of bytes with length equal to [`u32`]
    pub const fn ptr() -> Self {
        Self(PTR_SIZE)
    }

    /// Creates a representation of bytes with length equal to [`i8`]
    pub const fn byte() -> Self {
        Self(1)
    }

    /// Creates a representation of bytes with zero length
    pub const fn null() -> Self {
        Self(0)
    }

    /// Creates a stack ptr to init position
    pub const fn start() -> Self {
        Self(0)
    }
}

impl From<Bytes> for imuc_ast::prim::Integer {
    fn from(value: Bytes) -> Self {
        Self::I32(value.0 as i32)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum NumBytes {
    I8,
    I16,
    I32,
    I64,
}

impl NumBytes {
    /// Creates the number of bytes corresponding to a pointer
    pub const fn ptr() -> Self {
        // WARN: Please change this when pointer size changes
        Self::I32
    }
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

use imuc_lexer::token::ResTy;
impl TryFrom<ResTy> for NumBytes {
    type Error = Error;
    fn try_from(value: ResTy) -> Result<NumBytes> {
        match value {
            ResTy::Bool | ResTy::I8 => Ok(NumBytes::I8),
            ResTy::I16 => Ok(NumBytes::I16),
            ResTy::I32 | ResTy::F32 => Ok(NumBytes::I32),
            ResTy::I64 | ResTy::F64 => Ok(NumBytes::I64),
            ResTy::Ptr | ResTy::Str => Ok(PTR_BYTES),
            _ => Err(errors::IrError::NotSized(format!("{value:?}")).into()),
        }
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

impl From<NumBytes> for Bytes {
    fn from(value: NumBytes) -> Self {
        match value {
            NumBytes::I8 => Bytes::new(1),
            NumBytes::I16 => Bytes::new(2),
            NumBytes::I32 => Bytes::new(4),
            NumBytes::I64 => Bytes::new(8),
        }
    }
}
