use crate::prelude::*;
use imuc_ir::{
    io::{IrReader, Rw},
    module::Header,
};
use std::collections::{hash_map::Entry, HashMap};
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::{Arc, RwLock};

type ImportsMap = HashMap<StrRef, StrRef>;

/// A map from handy aliases to imported functions to actual function names
///
/// This is included in a body, and will be effective in every body under that body in stack order.
/// To implement this, backward pointers are used, which means you shouldn't delete the body prior
/// to the current body containing this map
#[derive(Debug, Clone, Default)]
pub(crate) struct Imports {
    map: ImportsMap,
    prev: Option<NonNull<Imports>>,
}

impl Imports {
    /// Creates a new, empty imports map under a given map, which you cannot delete before this
    /// map, you also shouldn't create any loops when passing the pointer
    pub(crate) fn new_under(prev: &mut Imports) -> Imports {
        Self {
            map: Default::default(),
            prev: Some(NonNull::from(prev)),
        }
    }

    /// Queries the name from bottom to top, returning the first that match, if any.
    /// This is unsafe because pointer access is performed
    pub(crate) unsafe fn query(&self, name: impl AsRef<str>) -> Option<StrRef> {
        let mut this = Some(NonNull::from(self));
        while let Some(ptr) = this {
            let imports = unsafe { ptr.as_ref() };
            if let Some(name) = imports.map.get(name.as_ref()) {
                return Some(name.clone());
            }
            this = imports.prev;
        }
        None
    }

    /// Inserts a new alias to the current layer of imports, if the key is not occupied, or no
    /// action will be done. Returns whether insertion is successful
    pub(crate) fn insert(&mut self, alias: StrRef, value: StrRef) -> bool {
        debug!("Insert alias: {} -> {}", alias, value);

        match self.map.entry(alias) {
            Entry::Occupied(_) => false,
            Entry::Vacant(vacant) => {
                vacant.insert(value);
                true
            }
        }
    }
}

/// The locked shared handle for [`ImportPool`]
pub type ImportPoolHandle = Arc<RwLock<ImportPool>>;

/// A cache pool for all modules read externally and can be used on repeat
#[derive(Default)]
pub struct ImportPool {
    pool: HashMap<PathBuf, Arc<ImportCache>>,
}

/// A single cache of a loaded module, containing its header info
pub struct ImportCache {
    pub header: imuc_ir::module::Header,
}

impl ImportPool {
    /// Attempts to read from cache, or read a new file and compiles it.
    /// Returns any errors encounter during io and compilation.
    pub fn load(&mut self, file: impl Into<PathBuf>) -> Result<Arc<ImportCache>> {
        let file = file.into();
        let path = file.with_file_name("lib.iuh");
        match self.pool.entry(file) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let reader = std::io::BufReader::new(
                    std::fs::OpenOptions::new()
                        .read(true)
                        .open(path.as_path())?,
                );
                let reader = IrReader::new(reader, true);
                let header = Header::read(reader)?;
                Ok(entry.insert(Arc::new(ImportCache { header })).clone())
            }
        }
    }
}
