use crate::prelude::*;
use crate::sym::*;
use std::collections::BTreeMap;

/// Header for a module, defining interfaces of types and functions
///
/// This is used when referencing to a external module. For compiling new modules, [`Module`](`crate::module::Module`) is used
pub struct Header {
    pub ty: BTreeMap<StrRef, Ty>,
    pub fun: BTreeMap<StrRef, FunSig>,
}

impl Rw for Header {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let mut ty = Vec::new();
        loop {
            if input.peek_line_or_else()? == "%" {
                break;
            }
            ty.push(Ty::read(&mut input)?);
        }
        let ty = BTreeMap::from_iter(ty.into_iter().map(|ty| (ty.name.clone(), ty)));

        let mut fun = Vec::new();
        loop {
            if input.peek_line_or_else()? == "%" {
                break;
            }
            let name = StrRef::from(input.read_until(' ')?);
            fun.push((name, FunSig::read(&mut input)?));
        }
        let fun = BTreeMap::from_iter(fun);

        Ok(Header { ty, fun })
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        for ty in self.ty.values() {
            write!(output, "{} ", &*ty.name)?;
            ty.write(&mut output)?;
            writeln!(output)?;
        }
        writeln!(output, "%")?;
        for (name, fun) in self.fun.iter() {
            writeln!(output, "{}", &**name)?;
            fun.write(&mut output)?;
            writeln!(output)?;
        }
        writeln!(output, "%")?;
        Ok(())
    }
}
