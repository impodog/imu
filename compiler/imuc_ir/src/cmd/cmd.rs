use super::{Bytes, NumBytes, Ptr};
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

macro_rules! arithmetic {
    (read $name: ident, $bytes: ident, $input: ident) => {{
        let bytes = $bytes.try_into()?;
        let lhs = Ptr::read(&mut $input)?;
        let rhs = Ptr::read(&mut $input)?;
        Ok(Self::$name(bytes, lhs, rhs))
    }};
    (write $name: literal, $bytes: ident, $lhs: ident, $rhs: ident, $output: ident) => {{
        write!($output, concat!($name, "{} "), char::from(*$bytes))?;
        $lhs.write(&mut $output)?;
        write!($output, " ")?;
        $rhs.write(&mut $output)?;
    }};
}

#[derive(Clone)]
pub enum Cmd {
    Dupli(Bytes, Ptr),
    Store(crate::sym::Prim),
    Wrap(Bytes, Ptr),
    Not(NumBytes, Ptr),
    Add(NumBytes, Ptr, Ptr),
    Sub(NumBytes, Ptr, Ptr),
    Mul(NumBytes, Ptr, Ptr),
    Div(NumBytes, Ptr, Ptr),
    Or(NumBytes, Ptr, Ptr),
    And(NumBytes, Ptr, Ptr),
    Xor(NumBytes, Ptr, Ptr),
    EqI8(Ptr, i8),
    Test(NumBytes, Ptr, Ptr),
    Addf(NumBytes, Ptr, Ptr),
    Subf(NumBytes, Ptr, Ptr),
    Mulf(NumBytes, Ptr, Ptr),
    Divf(NumBytes, Ptr, Ptr),
    Testf(NumBytes, Ptr, Ptr),
    /// Note that this command should not appear in [`CmdBody`]. It is only used to mark function ends in files,
    /// or to act as a placeholder for optional commands
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
            "str" => {
                let prim = crate::sym::Prim::read(&mut input)?;
                Ok(Self::Store(prim))
            }
            "wrp" => {
                let bytes = Bytes::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Wrap(bytes, ptr))
            }
            "not" => {
                let bytes = bytes.try_into()?;
                let opd = Ptr::read(&mut input)?;
                Ok(Self::Not(bytes, opd))
            }
            "add" => arithmetic!(read Add, bytes, input),
            "sub" => arithmetic!(read Sub, bytes, input),
            "mul" => arithmetic!(read Mul, bytes, input),
            "div" => arithmetic!(read Div, bytes, input),
            "bor" => arithmetic!(read Or, bytes, input),
            "and" => arithmetic!(read And, bytes, input),
            "xor" => arithmetic!(read Xor, bytes, input),
            "tst" => arithmetic!(read Test, bytes, input),
            "eql" => {
                let bytes = Bytes::read(&mut input)?;
                let value = input.read_until(' ')?.parse::<i8>()?;
                Ok(Self::EqI8(bytes, value))
            }
            "adf" => arithmetic!(read Addf, bytes, input),
            "sbf" => arithmetic!(read Subf, bytes, input),
            "mlf" => arithmetic!(read Mulf, bytes, input),
            "dvf" => arithmetic!(read Divf, bytes, input),
            "tsf" => arithmetic!(read Testf, bytes, input),
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
            Self::Store(prim) => {
                write!(output, "str ")?;
                prim.write(&mut output)?;
            }
            Self::Wrap(bytes, ptr) => {
                write!(output, "wrp ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                ptr.write(&mut output)?;
            }
            Self::Not(bytes, opd) => {
                write!(output, "not{} ", char::from(*bytes))?;
                write!(output, " ")?;
                opd.write(&mut output)?;
            }
            Self::Add(bytes, lhs, rhs) => arithmetic!(write "add", bytes, lhs, rhs, output),
            Self::Sub(bytes, lhs, rhs) => arithmetic!(write "sub", bytes, lhs, rhs, output),
            Self::Mul(bytes, lhs, rhs) => arithmetic!(write "mul", bytes, lhs, rhs, output),
            Self::Div(bytes, lhs, rhs) => arithmetic!(write "div", bytes, lhs, rhs, output),
            Self::Or(bytes, lhs, rhs) => arithmetic!(write "bor", bytes, lhs, rhs, output),
            Self::And(bytes, lhs, rhs) => arithmetic!(write "and", bytes, lhs, rhs, output),
            Self::Xor(bytes, lhs, rhs) => arithmetic!(write "xor", bytes, lhs, rhs, output),
            Self::EqI8(opd, value) => {
                write!(output, "eql ")?;
                opd.write(&mut output)?;
                write!(output, " {}", value)?;
            }
            Self::Addf(bytes, lhs, rhs) => arithmetic!(write "adf", bytes, lhs, rhs, output),
            Self::Subf(bytes, lhs, rhs) => arithmetic!(write "sbf", bytes, lhs, rhs, output),
            Self::Mulf(bytes, lhs, rhs) => arithmetic!(write "mlf", bytes, lhs, rhs, output),
            Self::Divf(bytes, lhs, rhs) => arithmetic!(write "dvf", bytes, lhs, rhs, output),
            Self::Testf(bytes, lhs, rhs) => arithmetic!(write "tsf", bytes, lhs, rhs, output),
            Self::Test(bytes, lhs, rhs) => arithmetic!(write "tst", bytes, lhs, rhs, output),
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
            writeln!(output)?;
        }
        Cmd::End.write(&mut output)?;
        writeln!(output)?;
        Ok(())
    }
}
