use crate::file::file::*;
use crate::prelude::*;
use imuc_lexer::Filename;
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};

/// A map from absolute paths to file handles, providing access and pretty error formatting
#[derive(Default)]
pub struct FileMap {
    map: HashMap<PathBuf, Arc<FileHandle>>,
}

impl FileMap {
    /// Queries a file name from the map, or creates one if not already created,
    /// returning any file access errors
    pub fn query(&mut self, filename: Filename) -> Result<Arc<FileHandle>> {
        let path = Path::new(filename.get().as_str()).canonicalize()?;
        match self.map.entry(path) {
            Entry::Occupied(occupied) => Ok(occupied.get().clone()),
            Entry::Vacant(vacant) => {
                let result = vacant.insert(Arc::new(FileHandle::new(filename))).clone();
                Ok(result)
            }
        }
    }
}

pub static FILE_MAP: LazyLock<RwLock<FileMap>> = LazyLock::new(Default::default);
