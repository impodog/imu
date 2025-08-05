use crate::{ParserInput, ParserSequence, TokenKindSet};
use imuc_ast::name::Prefix;
use imuc_error::*;
use imuc_lexer::TokenKind;
use std::collections::VecDeque;

/// A parser that iterates over sequences of `ParserInput`, with syntax trees
pub struct Parser<'s, I>
where
    I: ParserSequence<'s>,
{
    seq: I,
    queue: VecDeque<ParserStack<'s>>,
    prefixes: VecDeque<Prefix>,
    pub look_up: imuc_ast::name::LookUp,
    pub resolver: imuc_path::Resolver,
    _phantom: std::marker::PhantomData<&'s str>,
}

struct ParserStack<'s> {
    input: ParserInput<'s>,
    cursor: imuc_lexer::Cursor,
    file_info: crate::file::FileInfo,
}

impl<'s, I> Parser<'s, I>
where
    I: ParserSequence<'s>,
{
    /// Creates a parser with underlying sequence given
    pub fn new(seq: impl IntoIterator<Item = ParserInput<'s>, IntoIter = I>) -> Self {
        Self {
            seq: seq.into_iter(),
            queue: VecDeque::with_capacity(2),
            prefixes: VecDeque::with_capacity(1),
            look_up: Default::default(),
            resolver: Default::default(),
            _phantom: Default::default(),
        }
    }

    /// Inserts a prefix into the front queue, blocking any other queue operations by returning dummy tokens
    /// until the prefix is collected by `Self::pop_prefix`
    pub fn push_prefix(&mut self, prefix: imuc_ast::name::Prefix) {
        self.prefixes.push_back(prefix);
    }

    /// Acquires the prefix in the queue, if any, otherwise no action is done.
    /// See `Self::push_prefix`
    pub fn pop_prefix(&mut self) -> Option<imuc_ast::name::Prefix> {
        self.prefixes.pop_front()
    }

    /// Returns if there is any prefix in the queue
    pub fn has_prefix(&self) -> bool {
        !self.prefixes.is_empty()
    }

    /// Returns the nth pending result of `Self::next_token` without consuming the token,
    /// index starting from 0
    ///
    /// Errors are only caused by lexer errors
    /// Note that peeking does not change current `Self::relative_cursor`] and [`Self::file_info`
    pub fn peek_nth(&mut self, index: usize) -> Result<Option<ParserInput<'s>>> {
        // The required length for the queue to contain the index
        let len = index + 1;
        while self.queue.len() < len {
            // Call directly because the queue lack new tokens, and must be retrieved from the seq
            if let Some(token) = self.next_token_directly()? {
                self.queue.push_back(token);
            } else {
                return Ok(None);
            }
        }
        if let Some(input) = self.queue.get(index) {
            Ok(Some(input.input))
        } else {
            Ok(None)
        }
    }

    /// Returns the next pending result of `Self::next_token` without consuming the token
    ///
    /// Errors are only caused by lexer errors
    /// Note that peeking does not change current `Self::relative_cursor`] and [`Self::file_info`
    pub fn peek(&mut self) -> Result<Option<ParserInput<'s>>> {
        if let Some(input) = self.queue.front() {
            Ok(Some(input.input))
        } else if let Some(token) = self.next_token_unfiltered()? {
            self.queue.push_back(token);
            Ok(Some(
                self.queue
                    .back()
                    .expect("Should contain an element after pushing")
                    .input,
            ))
        } else {
            Ok(None)
        }
    }

    /// Returns the next pending result of `Self::next_token`] if its matches the given kinds, or [`Ok(None)` is returned
    ///
    /// Errors are only caused by lexer errors
    pub fn next_if(
        &mut self,
        kind: &impl for<'a> TokenKindSet<'a>,
    ) -> Result<Option<ParserInput<'s>>> {
        let input = self.peek()?;
        if input.is_some_and(|input| kind.contains(&input.kind)) {
            self.queue
                .pop_front()
                .expect("Should contain an element after peeking one");
            self.peek()?;
            Ok(input)
        } else {
            Ok(None)
        }
    }

    /// Gets the next token, if any, while mapping the possible errors
    /// If the token is an error, an `Err` result is returned
    pub fn next_token(&mut self) -> Result<Option<ParserInput<'s>>> {
        self.filter_useless()?;
        let result = self.next_token_unfiltered()?;
        self.filter_useless()?;
        Ok(result.map(|result| result.input))
    }

    /// Internal function of `Self::next_token`, filters any useless tokens
    fn filter_useless(&mut self) -> Result<()> {
        while self.peek()?.is_some_and(|input| {
            matches!(
                input.kind,
                TokenKind::Spacing(_) | TokenKind::Comment(_) | TokenKind::Stray
            )
        }) {
            self.queue
                .pop_front()
                .expect("Should contain an element after peeking one");
        }
        Ok(())
    }

    /// Internal function of `Self::next_token`, but does not filter tail useless tokens,
    /// and does not touch the queue, but get a new token directly from the seq
    fn next_token_directly(&mut self) -> Result<Option<ParserStack<'s>>> {
        let mut cursor = self.seq.relative_cursor();
        let mut file_info = self.seq.file_info();
        loop {
            let input = self.seq.next();
            if let Some(input) = input {
                if let TokenKind::LexError(error) = input.kind {
                    return self.error(errors::LexerError(error));
                } else if matches!(
                    input.kind,
                    TokenKind::Spacing(_) | TokenKind::Comment(_) | TokenKind::Stray
                ) {
                    cursor = self.seq.relative_cursor();
                    file_info = self.seq.file_info();
                } else {
                    return Ok(Some(ParserStack {
                        input,
                        cursor,
                        file_info,
                    }));
                }
            } else {
                return Ok(input.map(move |input| ParserStack {
                    input,
                    cursor,
                    file_info,
                }));
            }
        }
    }

    /// Internal function of `Self::next_token`, but does not filter tail useless tokens
    fn next_token_unfiltered(&mut self) -> Result<Option<ParserStack<'s>>> {
        if let Some(front) = self.queue.pop_front() {
            return Ok(Some(front));
        }
        self.next_token_directly()
    }

    /// Gets the next token, mapping the errors, while regarding EOF as an error
    pub fn next_some(&mut self) -> Result<ParserInput<'s>> {
        self.next_token()
            .and_then(|input| input.ok_or_else(|| self.map_err(errors::SyntaxError::ExpectedAny)))
    }

    /// Gets the next token, mapping the errors, and takes any token other than the required kind as errors
    ///
    /// You should use this when you are sure to consume one token of the desired token kind
    pub fn next_expected(
        &mut self,
        kind: &impl for<'a> TokenKindSet<'a>,
    ) -> Result<ParserInput<'s>> {
        self.next_some().and_then(|input| {
            if !kind.contains(&input.kind) {
                let expect = {
                    use std::fmt::Write;
                    let mut expect = String::new();
                    kind.to_iter().enumerate().for_each(|(index, token)| {
                        if index > 0 {
                            write!(&mut expect, ",")
                                .expect("formatting error message should not fail");
                        }
                        write!(&mut expect, "{token:?}")
                            .expect("formatting error message should not fail");
                    });
                    expect
                };
                self.error(imuc_error::errors::SyntaxError::Expected {
                    expect,
                    found: input.kind,
                })
            } else {
                Ok(input)
            }
        })
    }

    /// Maps the error with appropriate context, outputting the new error
    pub fn map_err(&self, err: impl Into<Error>) -> Error {
        self.seq.map_error(err.into())
    }

    /// Gets the current file info, with the cursor same as `Self::relative_cursor`
    pub fn file_info(&self) -> crate::file::FileInfo {
        if let Some(front) = self.queue.front() {
            front.file_info
        } else {
            self.seq.file_info()
        }
    }

    /// Gets the current relative pointer, which is guaranteed to be increasing in position
    /// Note that this is different from what internal seq returns, as the parser can peek one
    /// token ahead
    pub fn relative_cursor(&self) -> imuc_lexer::Cursor {
        if let Some(front) = self.queue.front() {
            front.cursor
        } else {
            self.seq.relative_cursor()
        }
    }

    /// Maps the error then output a `Result`] of [`Err`
    pub fn error<R>(&self, err: impl Into<Error>) -> Result<R> {
        Err(self.map_err(err))
    }

    /// Returns an error if the lexer raises one, or return whether the lexer is exhausted
    pub fn is_empty(&mut self) -> Result<bool> {
        if self.queue.is_empty() {
            Ok(self.peek()?.is_none())
        } else {
            Ok(false)
        }
    }
}
