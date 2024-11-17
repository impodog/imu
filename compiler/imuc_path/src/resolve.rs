use crate::prelude::*;
use std::borrow::Cow;
use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

/// Resolves module paths starting from an array of base paths
#[derive(Default)]
pub struct Resolver {
    set: BTreeSet<PathBuf>,
    cache: HashMap<String, Arc<crate::Module>>,
}

impl Resolver {
    /// Creates an empty resolver
    pub fn new() -> Self {
        Self::default()
    }

    /// Querys the module name from the resolver
    ///
    /// The path of the return module, if any, is guaranteed to be an existing directory
    pub fn query(&mut self, module: Cow<str>) -> Result<Arc<crate::Module>> {
        if let Some(module) = self.cache.get(module.as_ref()) {
            return Ok(module.clone());
        }
        for base in self.set.iter() {
            let path = base.join(module.as_ref());
            if path.exists() && path.is_dir() {
                let path = path.canonicalize()?;
                let module = self
                    .cache
                    .entry(module.into_owned())
                    .or_insert(Arc::new(crate::Module::new(path).resolve()));
                return Ok(module.clone());
            }
        }
        Err(errors::PathError::ModuleNotFound(module.into_owned()).into())
    }

    /// Tries to insert a base path, returns Err if the path is not resolvable
    pub fn insert(&mut self, path: &Path) -> Result<()> {
        self.set.insert(path.canonicalize()?);
        Ok(())
    }
}
