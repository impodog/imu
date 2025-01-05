use crate::*;

#[derive(Debug, Error)]
pub enum ConvError {
    #[error("undefined value: {0}")]
    UndefinedValue(String),
    #[error("undefined type: {0}")]
    UndefinedType(String),
    #[error("uninitialized type: {0}")]
    UninitializedType(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("unknown field: {0}")]
    UnknownField(String),
    #[error("types mismatch: {0}")]
    TypesMismatch(String),
    #[error("'Self' type context if required")]
    SelfRequired,
    #[error("a value is required in {0}")]
    ValueRequired(String),
    #[error("a primitive is required in {0}")]
    PrimitiveRequired(String),
    #[error("a type is required in {0}")]
    TypeRequired(String),
    #[error("a structure is required in {0}")]
    CusRequired(String),
}
