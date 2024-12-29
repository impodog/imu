use crate::*;

#[derive(Debug, Error)]
pub enum ConvError {
    #[error("undefined value: {0}")]
    UndefinedValue(String),
    #[error("undefined type: {0}")]
    UndefinedType(String),
    #[error("uninitialized type: {0}")]
    UninitializedType(String),
    #[error("a function is required by this operation")]
    FunctionRequired,
    #[error("a value is required in {0}")]
    ValueRequired(String),
}
