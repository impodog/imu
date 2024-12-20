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
    #[error("expected {expect:?}")]
    ExpectedToken { expect: TokenKind },
    #[error("expected a token, found EOF")]
    ExpectedAny,
    #[error("item type {item:?} does not match with alias type {alias:?}")]
    AliasMismatch { item: TokenKind, alias: TokenKind },
    #[error("unknown escape sequence")]
    UnknownEscape,
    #[error("too many operators in an expression")]
    TooManyOp,
    #[error("too few operators in an expression")]
    TooFewOp,
}
