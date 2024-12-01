use imuc_lexer::token::ResTy;
use std::ops::{Add, Deref};

/// Representation of the length of a single type
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Bytes(u32);
impl Bytes {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn zero() -> Self {
        Self(0)
    }
}
impl Deref for Bytes {
    type Target = u32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl From<u32> for Bytes {
    fn from(value: u32) -> Self {
        Self(value)
    }
}
impl Into<u32> for Bytes {
    fn into(self) -> u32 {
        self.0
    }
}
impl Add<Bytes> for Bytes {
    type Output = Bytes;
    fn add(self, rhs: Bytes) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl TryFrom<ResTy> for Bytes {
    type Error = ResTy;
    fn try_from(value: ResTy) -> Result<Self, Self::Error> {
        let count = match value {
            ResTy::I8 => 8,
            ResTy::I16 => 16,
            ResTy::I32 => 32,
            ResTy::I64 => 64,
            ResTy::I128 => 128,
            ResTy::F32 => 32,
            ResTy::F64 => 64,
            ResTy::Ptr | ResTy::Str => crate::consts::PTR_BYTES,
            _ => return Err(value),
        };
        Ok(Self(count))
    }
}
