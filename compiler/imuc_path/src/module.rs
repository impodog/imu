use crate::prelude::*;
use std::{collections::HashMap, os::unix::ffi::OsStrExt};
use walkdir::WalkDir;

pub enum SubModule {
    File(crate::File),
    Module(Module),
}

pub struct Module {
    base: PathBuf,
    sub: HashMap<String, SubModule>,
}

impl Module {
    pub fn new(base: PathBuf) -> Self {
        Self {
            base,
            sub: Default::default(),
        }
    }

    pub fn resolve(mut self) -> Self {
        for entry in WalkDir::new(&self.base)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(|entry| entry.ok())
        {
            if entry.path().is_dir() {
                let name = String::from_utf8_lossy(entry.file_name().as_bytes()).into_owned();
                let module = Module::new(entry.into_path()).resolve();
                self.sub.insert(name, SubModule::Module(module));
            } else if entry.path().is_file() {
                let name = String::from_utf8_lossy(entry.file_name().as_bytes()).into_owned();
                let file = crate::File::new(entry.into_path());
                self.sub.insert(name, SubModule::File(file));
            }
        }
        self
    }

    /// Returns the corresponding submodule of the name
    pub fn get(&self, name: &str) -> Option<&SubModule> {
        self.sub.get(name)
    }

    /// Returns a iterator over submodules and their names
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, SubModule> {
        self.sub.iter()
    }

    /// Returns the base path of the module
    pub fn base(&self) -> &Path {
        &self.base
    }
}
