use crate::prelude::*;

/// A function containing its name and commands
pub struct Fun {
    pub name: StrRef,
    pub sig: FunSig,
    pub body: crate::cmd::CmdBody,
}

/// Function without signature, usually stored in a source file next to a header file
pub struct SiglessFun {
    pub name: StrRef,
    pub body: crate::cmd::CmdBody,
}

impl From<Fun> for (FunSig, SiglessFun) {
    fn from(value: Fun) -> Self {
        let Fun { name, sig, body } = value;
        (sig, SiglessFun { name, body })
    }
}

impl From<(FunSig, SiglessFun)> for Fun {
    fn from((sig, SiglessFun { name, body }): (FunSig, SiglessFun)) -> Self {
        Self { name, sig, body }
    }
}

/// The signature of a function, containing the parameter and return type, and templates
pub struct FunSig {
    pub param: super::Ty,
    pub ret: super::Ty,
}

impl Rw for Fun {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = StrRef::from(input.read_line()?);
        let sig = FunSig::read(&mut input)?;
        let body = crate::cmd::CmdBody::read(&mut input)?;
        Ok(Self { name, sig, body })
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        write!(output, "{}\n", &*self.name)?;
        self.sig.write(&mut output)?;
        self.body.write(&mut output)?;
        Ok(())
    }
}

impl Rw for SiglessFun {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = StrRef::from(input.read_line()?);
        let body = crate::cmd::CmdBody::read(&mut input)?;
        Ok(Self { name, body })
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        write!(output, "{}\n", &*self.name)?;
        self.body.write(&mut output)?;
        Ok(())
    }
}

impl Rw for FunSig {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let param = super::Ty::read(&mut input)?;
        let ret = super::Ty::read(&mut input)?;
        Ok(Self { param, ret })
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        self.param.write(&mut output)?;
        write!(output, "\n")?;
        self.ret.write(&mut output)?;
        Ok(())
    }
}
