use crate::*;
use imuc_lexer::TokenKind;

#[derive(Debug, Error)]
pub enum SyntaxError {
    #[error("expected {expect:?}, found {found:?}")]
    Expected { expect: String, found: TokenKind },
    #[error("expected a token, found EOF")]
    ExpectedAny,
    #[error("unknown escape sequence")]
    UnknownEscape,
}
