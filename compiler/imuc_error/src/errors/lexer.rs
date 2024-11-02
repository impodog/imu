use crate::*;

#[derive(Debug, Error)]
#[error("lexer: {0:?}")]
pub struct LexerError(imuc_lexer::token::LexError);
