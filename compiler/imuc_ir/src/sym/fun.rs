use crate::prelude::*;
use std::collections::BTreeSet;

/// A function containing its name and commands
pub struct Fun {
    pub name: StrRef,
    pub sig: FunSig,
    pub body: crate::cmd::CmdBody,
}

/// The signature of a function, containing the parameter and return type, and templates
pub struct FunSig {
    pub templ: BTreeSet<super::TemplId>,
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

impl Rw for FunSig {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let mut templ = Vec::new();
        let ids = input.read_line()?.split(' ');
        for id in ids {
            let id = id.parse::<super::TemplId>()?;
            templ.push(id);
        }
        let templ = BTreeSet::from_iter(templ.into_iter());
        let param = super::Ty::read(&mut input)?;
        let ret = super::Ty::read(&mut input)?;
        Ok(Self { templ, param, ret })
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        for (is_last, id) in self
            .templ
            .iter()
            .enumerate()
            .map(|(i, id)| (i + 1 == self.templ.len(), id))
        {
            write!(output, "{}", id)?;
            if !is_last {
                write!(output, " ")?;
            }
        }
        write!(output, "\n")?;
        self.param.write(&mut output)?;
        write!(output, "\n")?;
        self.ret.write(&mut output)?;
        Ok(())
    }
}
