use crate::*;

#[derive(Debug, Error)]
pub enum IrError {
    #[error("unmatched {0:?} and {1:?}")]
    Unmatched(char, char),
    #[error("type {0} is not allowed in IR")]
    TypeNotAllowed(String),
    #[error("no such command modifier: {0}")]
    NoSuchCommandMod(String),
    #[error("no such command: {0}")]
    NoSuchCommand(String),
    #[error("no such primitive: {0}")]
    NoSuchPrimitive(char),
    #[error("no such type: {0}")]
    NoSuchType(String),
    #[error("no such value: {0}")]
    NoSuchValue(String),
    #[error("unknown escape sequence")]
    UnknownEscape(String),
    #[error("missing function signature: {0}")]
    MissingSignature(String),
    #[error("unimplemented function signature: {0}")]
    UnimplementedSignature(String),
    #[error("unexpected eof hit")]
    Eof,
    #[error("internal type required for IR generation")]
    InternalRequired,
    #[error("character {0:?} and its surrounding values are required")]
    CharRequired(char),
    #[error("looped reference of types")]
    LoopedReference,
}
