use crate::*;
use imuc_lexer::TokenKind;

#[derive(Debug, Error)]
pub enum SyntaxError {
    #[error("expected {expect}, found {found:?}")]
    Expected { expect: String, found: TokenKind },
    #[error("expected {expect} after {after:?}")]
    ExpectedAfter { expect: String, after: TokenKind },
    #[error("expected {expect} in {context}")]
    ExpectedIn { expect: String, context: String },
    #[error("expected a token, found EOF")]
    ExpectedAny,
    #[error("unknown escape sequence")]
    UnknownEscape,
}
