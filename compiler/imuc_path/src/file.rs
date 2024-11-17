use crate::prelude::*;

#[derive(Clone)]
pub struct File {
    path: PathBuf,
}

impl File {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Returns the base path of the file
    pub fn path(&self) -> &Path {
        &self.path
    }
}
