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

#[derive(Clone, Copy)]
pub enum Cmd {
    Dupli(Bytes, Ptr),
    /// Note that this command should not appear in [`CmdBody`]. It is only used to mark function ends in files
    End,
}

impl Rw for Cmd {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let cmd = input.read_until(' ')?;
        match cmd {
            "dup" => {
                let bytes = Bytes::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Dupli(bytes, ptr))
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
                ptr.write(&mut output)?;
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
