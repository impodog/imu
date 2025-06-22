use crate::prelude::*;
use std::collections::HashMap;
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
                let subdir_entry = entry.path().join("mod.iu");
                if subdir_entry.exists() {
                    let name = String::from_utf8_lossy(
                        entry
                            .path()
                            .file_name()
                            .expect("Should be a dir after checking")
                            .as_encoded_bytes(),
                    )
                    .into_owned();
                    let module = Module::new(entry.path().to_path_buf()).resolve();
                    self.sub.insert(name, SubModule::Module(module));
                }
            } else if entry.path().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "iu")
            {
                let name = String::from_utf8_lossy(
                    entry
                        .path()
                        .file_stem()
                        .expect("should be a file after checking")
                        .as_encoded_bytes(),
                )
                .into_owned();
                let file = crate::File::new(
                    entry
                        .into_path()
                        .canonicalize()
                        .expect("existing path should not fail to canonicalize"),
                );
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
