use crate::*;

#[derive(Debug, Error)]
#[error("lexer: {0:?}")]
pub struct LexerError(pub imuc_lexer::token::LexError);
