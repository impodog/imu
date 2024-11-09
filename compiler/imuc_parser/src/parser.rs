use crate::{ParserInput, ParserSequence, TokenKindSet};
use imuc_error::*;
use imuc_lexer::TokenKind;

/// A parser that iterates over sequences of [`ParserInput`], with syntax trees
pub struct Parser<'s, I>
where
    I: ParserSequence<'s>,
{
    seq: I,
    stack: Option<ParserInput<'s>>,
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
            stack: None,
            _phantom: Default::default(),
        }
    }

    /// Returns the next pending result of [`Self::next_token`] without consuming the token
    ///
    /// Errors are only caused by lexer errors
    pub(crate) fn peek(&mut self) -> Result<Option<ParserInput<'s>>> {
        if let Some(ref input) = self.stack {
            Ok(Some(*input))
        } else {
            let input = self.next_token()?;
            self.stack = input;
            Ok(input)
        }
    }

    /// Returns the next pending result of [`Self::next_token`] if its matches the given kinds, or [`Ok(None)`] is returned
    ///
    /// Errors are only caused by lexer errors
    pub(crate) fn next_if(
        &mut self,
        kind: &impl for<'a> TokenKindSet<'a>,
    ) -> Result<Option<ParserInput<'s>>> {
        let input = self.peek()?;
        if input.is_some_and(|input| kind.contains(&input.kind)) {
            Ok(input)
        } else {
            Ok(None)
        }
    }

    /// Gets the next token, if any, while mapping the possible errors
    /// If the token is an error, an [`Err`] result is returned
    pub(crate) fn next_token(&mut self) -> Result<Option<ParserInput<'s>>> {
        if let Some(input) = std::mem::take(&mut self.stack) {
            return Ok(Some(input));
        }

        let input = self.seq.next();
        if let Some(input) = input {
            if let TokenKind::LexError(error) = input.kind {
                self.error(errors::LexerError(error))
            } else {
                Ok(Some(input))
            }
        } else {
            Ok(input)
        }
    }

    /// Gets the next token, mapping the errors, while regarding EOF as an error
    pub(crate) fn next_some(&mut self) -> Result<ParserInput<'s>> {
        self.next_token()
            .and_then(|input| input.ok_or_else(|| self.map_error(errors::SyntaxError::ExpectedAny)))
    }

    /// Gets the next token, mapping the errors, and takes any token other than the required kind as errors
    pub(crate) fn next_expected_token(
        &mut self,
        kind: &impl for<'a> TokenKindSet<'a>,
    ) -> Result<ParserInput<'s>> {
        self.next_some().and_then(|input| {
            if !kind.contains(&input.kind) {
                let expect = {
                    use std::fmt::Write;
                    let mut expect = String::new();
                    kind.to_iter().for_each(|token| {
                        write!(&mut expect, "{:?}", token)
                            .expect("formatting error message should not fail");
                    });
                    expect
                };
                self.error(errors::SyntaxError::Expected {
                    expect,
                    found: input.kind,
                })
            } else {
                Ok(input)
            }
        })
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
