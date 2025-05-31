use crate::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Reads env var from IMUC_PATH and support querying for modules
pub struct PathVar {
    modules: Vec<PathBuf>,
}

impl PathVar {
    /// Reads env var from IMUC_PATH and stores them in the return value
    pub fn read() -> Self {
        let modules = std::env::var("IMUC_PATH")
            .unwrap_or_else(|err| {
                warn!("When reading env var \"IMUC_PATH\", {}", err);
                Default::default()
            })
            .split(':')
            .filter_map(|path| {
                let path = Path::new(path);
                if path.exists() {
                    Some(path.to_path_buf())
                } else {
                    warn!("In env var \"IMUC_PATH\", path {:?} does not exist", path);
                    None
                }
            })
            .collect();
        Self { modules }
    }

    /// Queries for any presence of the module under working directory, or under path vars
    pub fn query(&self, name: &str) -> Option<PathBuf> {
        {
            let path = Path::new(name);
            if path.exists() {
                return Some(path.to_path_buf());
            }
        }
        for dir in self.modules.iter() {
            let path = dir.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        None
    }
}

pub static PATH_VAR: LazyLock<PathVar> = LazyLock::new(PathVar::read);
