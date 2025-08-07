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

macro_rules! conversion {
    (read $name: ident, $bytes: ident, $input: ident) => {{
        if $bytes.len() != 2 {
            return Err(errors::IrError::TwoNumBytesRequired($bytes.to_owned()).into());
        }
        let mut bytes_chars = $bytes.chars();
        let lhs = bytes_chars.next().expect("len is exactly 2").try_into()?;
        let rhs = bytes_chars.next().expect("len is exactly 2").try_into()?;
        let ptr = Ptr::read(&mut $input)?;
        Ok(Self::$name(lhs, rhs, ptr))
    }};
    (write $name: literal, $lhs: ident, $rhs: ident, $ptr: ident, $output: ident) => {{
        write!(
            $output,
            concat!($name, "{}{} "),
            char::from(*$lhs),
            char::from(*$rhs)
        )?;
        $ptr.write(&mut $output)?;
    }};
}

#[derive(Clone)]
pub enum Cmd {
    /// Duplicate bytes from pointer to the top of stack
    Dupli(Bytes, Ptr),
    /// Duplicate bytes from the first pointer, then overwrite contents of the second pointer.
    /// If the two segments overlap, correct behavior is also guaranteed
    Overwrite(Bytes, Ptr, Ptr),
    Store(crate::sym::Prim),
    StorePtr(Ptr),
    /// Converts the local pointer to the pointer relative to the global stack
    StorePtrAsGlobal(Ptr),
    /// Shrinks the stack to given size, discarding memory after it. If the stack is
    /// smaller than the given size, no action will be performed
    Shrink(Ptr),
    /// Allocates bytes on the heap, putting the pointer on top of the stack
    Alloc(Bytes),
    /// Deallocates the heap chunk by the pointer stored
    Dealloc(Ptr),
    /// Writes bytes(1st) with offset(2nd) to the heap pointer stored(4th), from another local stack pointer(3rd)
    WriteHeap(Bytes, Bytes, Ptr, Ptr),
    /// Reads bytes(1st) with offset(2nd) from the heap pointer stored(3rd)
    ReadHeap(Bytes, Bytes, Ptr),
    /// Reads bytes(1st) with offset(2nd) from the stack pointer stored(3rd)
    ReadStack(Bytes, Bytes, Ptr),
    /// Moves the stack ptr forward without having to do anything
    Skip(Bytes),
    Not(NumBytes, Ptr),
    Neg(NumBytes, Ptr),
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
    IToI(NumBytes, NumBytes, Ptr),
    FToF(NumBytes, NumBytes, Ptr),
    IToF(NumBytes, NumBytes, Ptr),
    FToI(NumBytes, NumBytes, Ptr),
    Fill0(Bytes),
    /// Jump to a specific location if eached
    Jump(Ptr),
    /// If the 1-byte condition in pointer 1 is true, jump to pointer 2
    JumpIf(Ptr, Ptr),
    /// Call the function with top bytes plus a function pointer at the bottom;
    /// The enum argument is the bytes of arguments *without* the function pointer.
    /// Unlike other systems, the ordering of arguments is not reversed
    Call(Bytes),
    /// Globally links to the function, putting its handle(ptr-sized) on top of the global stack
    Link(StrRef),
    /// Does the link like `Self::Link` with additional argument and return size check
    LinkCheck(Bytes, Bytes, StrRef),
    /// Note that this command should not appear in `CmdBody`. It is only used to mark function ends in files,
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
            "ovw" => {
                let bytes = Bytes::read(&mut input)?;
                let src = Ptr::read(&mut input)?;
                let dst = Ptr::read(&mut input)?;
                Ok(Self::Overwrite(bytes, src, dst))
            }
            "str" => {
                let prim = crate::sym::Prim::read(&mut input)?;
                Ok(Self::Store(prim))
            }
            "spt" => {
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::StorePtr(ptr))
            }
            "sag" => {
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::StorePtrAsGlobal(ptr))
            }
            "srk" => {
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Shrink(ptr))
            }
            "alc" => {
                let bytes = Bytes::read(&mut input)?;
                Ok(Self::Alloc(bytes))
            }
            "dal" => {
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Dealloc(ptr))
            }
            "rdh" => {
                let bytes = Bytes::read(&mut input)?;
                let offset = Bytes::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::ReadHeap(bytes, offset, ptr))
            }
            "wrh" => {
                let bytes = Bytes::read(&mut input)?;
                let offset = Bytes::read(&mut input)?;
                let src = Ptr::read(&mut input)?;
                let dst = Ptr::read(&mut input)?;
                Ok(Self::WriteHeap(bytes, offset, src, dst))
            }
            "rds" => {
                let bytes = Bytes::read(&mut input)?;
                let offset = Bytes::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::ReadStack(bytes, offset, ptr))
            }
            "not" => {
                let bytes = bytes.try_into()?;
                let opd = Ptr::read(&mut input)?;
                Ok(Self::Not(bytes, opd))
            }
            "neg" => {
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
            "iti" => conversion!(read IToI, bytes, input),
            "ftf" => conversion!(read FToF, bytes, input),
            "itf" => conversion!(read IToF, bytes, input),
            "fti" => conversion!(read FToI, bytes, input),
            "fil" => {
                let bytes = Bytes::read(&mut input)?;
                Ok(Self::Fill0(bytes))
            }
            "jmp" => {
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::Jump(ptr))
            }
            "jif" => {
                let cond = Ptr::read(&mut input)?;
                let ptr = Ptr::read(&mut input)?;
                Ok(Self::JumpIf(cond, ptr))
            }
            "cal" => {
                let bytes = Bytes::read(&mut input)?;
                Ok(Self::Call(bytes))
            }
            "lnk" => {
                let name = input.read_until(' ')?;
                Ok(Self::Link(StrRef::from(name)))
            }
            "lck" => {
                let param = Bytes::read(&mut input)?;
                let ret = Bytes::read(&mut input)?;
                let name = input.read_until(' ')?;
                Ok(Self::LinkCheck(param, ret, StrRef::from(name)))
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
            Self::Overwrite(bytes, src, dst) => {
                write!(output, "ovw ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                src.write(&mut output)?;
                write!(output, " ")?;
                dst.write(&mut output)?;
            }
            Self::Store(prim) => {
                write!(output, "str ")?;
                prim.write(&mut output)?;
            }
            Self::StorePtr(ptr) => {
                write!(output, "spt ")?;
                ptr.write(&mut output)?;
            }
            Self::StorePtrAsGlobal(ptr) => {
                write!(output, "sag ")?;
                ptr.write(&mut output)?;
            }
            Self::Shrink(ptr) => {
                write!(output, "srk ")?;
                ptr.write(&mut output)?;
            }
            Self::Alloc(bytes) => {
                write!(output, "alc ")?;
                bytes.write(&mut output)?;
            }
            Self::Dealloc(ptr) => {
                write!(output, "dal ")?;
                ptr.write(&mut output)?;
            }
            Self::ReadHeap(bytes, offset, ptr) => {
                write!(output, "rdh ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                offset.write(&mut output)?;
                write!(output, " ")?;
                ptr.write(&mut output)?;
            }
            Self::WriteHeap(bytes, offset, src, dst) => {
                write!(output, "wrh ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                offset.write(&mut output)?;
                write!(output, " ")?;
                src.write(&mut output)?;
                write!(output, " ")?;
                dst.write(&mut output)?;
            }
            Self::ReadStack(bytes, offset, ptr) => {
                write!(output, "rds ")?;
                bytes.write(&mut output)?;
                write!(output, " ")?;
                offset.write(&mut output)?;
                write!(output, " ")?;
                ptr.write(&mut output)?;
            }
            Self::Not(bytes, opd) => {
                write!(output, "not{} ", char::from(*bytes))?;
                opd.write(&mut output)?;
            }
            Self::Neg(bytes, opd) => {
                write!(output, "neg{} ", char::from(*bytes))?;
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
                write!(output, " {value}")?;
            }
            Self::Addf(bytes, lhs, rhs) => arithmetic!(write "adf", bytes, lhs, rhs, output),
            Self::Subf(bytes, lhs, rhs) => arithmetic!(write "sbf", bytes, lhs, rhs, output),
            Self::Mulf(bytes, lhs, rhs) => arithmetic!(write "mlf", bytes, lhs, rhs, output),
            Self::Divf(bytes, lhs, rhs) => arithmetic!(write "dvf", bytes, lhs, rhs, output),
            Self::Testf(bytes, lhs, rhs) => arithmetic!(write "tsf", bytes, lhs, rhs, output),
            Self::Test(bytes, lhs, rhs) => arithmetic!(write "tst", bytes, lhs, rhs, output),
            Self::IToI(lhs, rhs, ptr) => conversion!(write "iti", lhs, rhs, ptr, output),
            Self::FToF(lhs, rhs, ptr) => conversion!(write "ftf", lhs, rhs, ptr, output),
            Self::IToF(lhs, rhs, ptr) => conversion!(write "itf", lhs, rhs, ptr, output),
            Self::FToI(lhs, rhs, ptr) => conversion!(write "fti", lhs, rhs, ptr, output),
            Self::Fill0(bytes) => {
                write!(output, "fil ")?;
                bytes.write(&mut output)?;
            }
            Self::Jump(ptr) => {
                write!(output, "jmp ")?;
                ptr.write(&mut output)?;
            }
            Self::JumpIf(cond, ptr) => {
                write!(output, "jif ")?;
                cond.write(&mut output)?;
                write!(output, " ")?;
                ptr.write(&mut output)?;
            }
            Self::Call(bytes) => {
                write!(output, "cal ")?;
                bytes.write(&mut output)?;
            }
            Self::Link(name) => {
                write!(output, "lnk {name}")?;
            }
            Self::LinkCheck(param, ret, name) => {
                write!(output, "lck ")?;
                param.write(&mut output)?;
                write!(output, " ")?;
                ret.write(&mut output)?;
                write!(output, " {name}")?;
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
            writeln!(output)?;
        }
        Cmd::End.write(&mut output)?;
        writeln!(output)?;
        Ok(())
    }
}
