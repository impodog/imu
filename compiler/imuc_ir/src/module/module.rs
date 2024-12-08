use crate::prelude::*;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::BufReader;
use std::path::Path;

/// A complete module with types, functions and the defintions
pub struct Module {
    // header's function signatures are reused to in the new field, thus becoming useless
    pub ty: BTreeMap<StrRef, crate::sym::Ty>,
    pub fun: BTreeMap<StrRef, crate::sym::Fun>,
}

impl Module {
    pub fn split(self) -> (super::Header, Vec<crate::sym::SiglessFun>) {
        let Module { ty, fun } = self;
        let mut fun_arr = Vec::new();
        let mut sig_arr = Vec::new();
        for (name, sig_fun) in fun.into_iter() {
            let (sig, fun) = sig_fun.into();
            sig_arr.push((name, sig));
            fun_arr.push(fun);
        }
        let header = super::Header {
            ty,
            fun: BTreeMap::from_iter(sig_arr),
        };
        (header, fun_arr)
    }

    /// Writes the module to the path, regardless of the file extension
    pub fn split_to_path(self, base_path: &Path) -> Result<()> {
        let (header, fun) = self.split();
        let header_path = base_path.with_extension(".iuh");
        let src_path = base_path.with_extension(".iuc");

        header.write(
            OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(header_path)?,
        )?;
        for fun in fun.into_iter() {
            fun.write(
                OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .open(&src_path)?,
            )?;
        }
        Ok(())
    }
}

pub struct ModuleBuilder<T>
where
    T: IrRead,
{
    pub header: crate::module::Header,
    pub src: T,
}

impl ModuleBuilder<crate::io::IrReader<BufReader<File>>> {
    /// The path to read the module from, regardless of the file name extension
    pub fn from_path(base_path: &Path, external: bool) -> Result<Self> {
        let header = base_path.with_extension("iuh");
        let reader = BufReader::new(OpenOptions::new().read(true).open(header)?);
        let header = crate::io::IrReader::new(reader, external);
        let header = crate::module::Header::read(header)?;

        let source = base_path.with_extension("iuc");
        let reader = BufReader::new(OpenOptions::new().read(true).open(source)?);
        let src = crate::io::IrReader::new(reader, external);
        Ok(Self { header, src })
    }
}

impl<T> TryFrom<ModuleBuilder<T>> for Module
where
    T: IrRead,
{
    type Error = Error;

    fn try_from(value: ModuleBuilder<T>) -> Result<Self> {
        let ModuleBuilder {
            mut header,
            mut src,
        } = value;
        let mut fun_arr = Vec::<(StrRef, crate::sym::Fun)>::new();
        while src.peek_line().is_some() {
            let fun = crate::sym::SiglessFun::read(&mut src)?;
            let sig = header
                .fun
                .remove(&fun.name)
                .ok_or_else(|| errors::IrError::MissingSignature(fun.name.to_string()))?;
            fun_arr.push((fun.name.clone(), (sig, fun).into()));
        }
        if let Some((name, _)) = header.fun.pop_first() {
            Err(errors::IrError::UnimplementedSignature(name.to_string()).into())
        } else {
            let fun = BTreeMap::from_iter(fun_arr.into_iter());
            let super::Header { ty, fun: _fun } = header;
            Ok(Self { ty, fun })
        }
    }
}
