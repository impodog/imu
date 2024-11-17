use crate::*;

#[derive(Debug, Error)]
pub enum PathError {
    #[error("unable to find module {0} in path")]
    ModuleNotFound(String),
    #[error("imcomplete or broken import path")]
    BrokenPath,
}
