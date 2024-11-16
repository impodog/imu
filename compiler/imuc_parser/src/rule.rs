use crate::{Parser, ParserSequence};
use imuc_error::*;

pub trait Rule {
    type Output;

    /// The main entry for parsing, returns a corresponding AST
    ///
    /// If any parsing error occurred, an [`Err`] is returned
    /// If the first peeked tokens cannot be parsed, an [`Ok(None)`] is returned
    /// If the parsing is successful, an [`Ok(Some)`] is returned
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>;
}
