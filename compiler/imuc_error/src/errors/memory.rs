use crate::*;

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("type {0} has infinite size")]
    Recursive(String),
    #[error("Self type should not appear here")]
    UnexpectedSelf,
    #[error("unknown type handle {0}")]
    UnknownHandle(String),
    #[error("the number of elements overflowed the threshold of {0}")]
    OverflowError(usize),
}
