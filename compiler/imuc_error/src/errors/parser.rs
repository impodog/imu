use crate::*;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("string tokens need to be properly wrapped in quotes")]
    QuoteError,
}
