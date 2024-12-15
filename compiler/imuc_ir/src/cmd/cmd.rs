use super::{Bytes, Ptr};
use crate::prelude::*;
use std::ops::Deref;

#[derive(Clone)]
pub struct CmdBody {
    list: Vec<Cmd>,
}

impl CmdBody {
    pub fn new(list: Vec<Cmd>) -> Self {
        Self { list }
    }
}

impl Deref for CmdBody {
    type Target = [Cmd];
    fn deref(&self) -> &Self::Target {
        &self.list
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

#[derive(Clone)]
pub enum Cmd {
    Dupli(Bytes, Ptr),
    Add(NumBytes, Ptr, Ptr),
    Store(crate::sym::Prim),
    /// Note that this command should not appear in [`CmdBody`]. It is only used to mark function ends in files
    End,
}

impl Rw for Cmd {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let cmd = input.read_until(' ')?;
        let (cmd, bytes) = cmd
            .split_at_checked(3)
            .ok_or_else(|| errors::IrError::NoSuchCommand(cmd.to_owned()))?;
        match cmd {
            "dup" => {
                let bytes = Bytes::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Dupli(bytes, ptr))
            }
            "add" => {
                let bytes = bytes.try_into()?;
                let lhs = Ptr::read(&mut input)?;
                let rhs = Ptr::read(&mut input)?;
                Ok(Self::Add(bytes, lhs, rhs))
            }
            "str" => {
                let prim = crate::sym::Prim::read(&mut input)?;
                Ok(Self::Store(prim))
            }
            "end" => Ok(Self::End),
            _ => Err(errors::IrError::NoSuchCommand(cmd.to_owned()).into()),
        }
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        match self {
            Self::Dupli(bytes, ptr) => {
                write!(output, "dup ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                ptr.write(&mut output)?;
            }
            Self::Add(bytes, lhs, rhs) => {
                write!(output, "add{} ", char::from(*bytes))?;
                write!(output, " ")?;
                lhs.write(&mut output)?;
                write!(output, " ")?;
                rhs.write(&mut output)?;
            }
            Self::Store(prim) => {
                write!(output, "str ")?;
                prim.write(&mut output)?;
            }
            Self::End => {
                write!(output, "end")?;
            }
        }
        Ok(())
    }
}

impl Rw for CmdBody {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let mut body = Vec::new();
        loop {
            let cmd = Cmd::read(&mut input)?;
            if matches!(cmd, Cmd::End) {
                break;
            }
            body.push(cmd);
        }
        Ok(Self::new(body))
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        for cmd in self.iter() {
            cmd.write(&mut output)?;
            write!(output, "\n")?;
        }
        Cmd::End.write(&mut output)?;
        write!(output, "\n")?;
        Ok(())
    }
}
