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
pub const GLOBAL_PTR_SIZE: u32 = std::mem::size_of::<GlobalPtr>() as u32;
pub const PTR_BYTES: NumBytes = number_to_bytes(PTR_SIZE).unwrap();
pub const GLOBAL_PTR_BYTES: NumBytes = number_to_bytes(GLOBAL_PTR_SIZE).unwrap();

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

    /// Creates a representation of bytes with length equal to [`u32`]
    pub const fn ptr() -> Self {
        Self(PTR_SIZE)
    }

    /// Creates a representation of bytes with length equal to [`u64`]
    pub const fn global_ptr() -> Self {
        Self(GLOBAL_PTR_SIZE)
    }

    /// Creates a representation of bytes with length equal to [`i8`]
    pub const fn byte() -> Self {
        Self(1)
    }

    /// Creates a stack ptr to init position
    pub const fn start() -> Self {
        Self(0)
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
            _ => Err(errors::IrError::NotSized(format!("{:?}", value)).into()),
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

/// Represents a pointer to a stack of current or previous functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalPtr(u64);

impl GlobalPtr {
    /// Creates a new global ptr to a specific stack(starting from the bottom) and a pointer to a
    /// value in the stack
    pub fn new(stack: u32, ptr: Bytes) -> Self {
        let stack = (stack as u64) << 32;
        Self(stack + ptr.0 as u64)
    }

    /// Extracts the stack position
    pub fn stack(&self) -> u32 {
        (self.0 >> 32) as u32
    }

    /// Extracts the pointer in the stack
    pub fn ptr(&self) -> u32 {
        (self.0 & 0xFFFFFFFF) as u32
    }
}

use std::fmt::{Display, Formatter, Result as FmtResult};
impl Display for GlobalPtr {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "GlobalPtr {{ stack: {}, ptr: {} }}",
            self.stack(),
            self.ptr()
        )
    }
}

impl Rw for GlobalPtr {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let value = input.read_until(' ')?;
        let value = value.parse::<u64>()?;
        Ok(Self(value))
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        write!(output, "{}", self.0)?;
        Ok(())
    }
}
