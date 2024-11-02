use imuc_error::*;
use imuc_lexer::TokenKind;

/// The direct input to parser, holding [`TokenKind`] and its corresponding string slice
#[derive(Debug, Clone, Copy)]
pub struct ParserInput<'s> {
    pub kind: TokenKind,
    pub value: &'s str,
}

/// Trait for types that can be applied to a parser
/// This requires an iterator over [`ParserInput`] and context info
pub trait ParserSequence<'s>: Iterator<Item = ParserInput<'s>> + Send + Sync {
    fn map_error(&self, err: Error) -> Error;
}

/// A parser that iterates over sequences of [`ParserInput`], with syntax trees
pub struct Parser<'s, I>
where
    I: ParserSequence<'s>,
{
    seq: I,
    _phantom: std::marker::PhantomData<&'s str>,
}

impl<'s, I> Parser<'s, I>
where
    I: ParserSequence<'s>,
{
    /// Creates a parser with underlying sequence given
    pub fn new(seq: impl IntoIterator<Item = ParserInput<'s>, IntoIter = I>) -> Self {
        Self {
            seq: seq.into_iter(),
            _phantom: Default::default(),
        }
    }

    /// Maps the error with appropriate context, outputting the new error
    pub fn map_error(&self, err: impl Into<Error>) -> Error {
        self.seq.map_error(err.into())
    }

    /// Maps the error then output a [`Result`] of [`Err`]
    pub fn error<R>(&self, err: impl Into<Error>) -> Result<R> {
        Err(self.map_error(err))
    }
}
