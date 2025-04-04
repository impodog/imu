use crate::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
/// A immutable path that represents a file, cloning using Arc
pub struct File {
    path: Arc<PathBuf>,
}

impl File {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: Arc::new(path.into()),
        }
    }

    /// Returns the base path of the file
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }
}
