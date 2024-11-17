use imuc_error::Error;
use imuc_lexer::{Token, TokenKind};
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::sync::Arc;

/// Clonable information of [`FileReader`] holding the file string and cursor position
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub file: Arc<str>,
    pub line: usize,
    pub column: usize,
}

/// An adaptor from token lexers to [`ParserSequence`](`crate::ParserSequence`)
pub struct FileReader<'s, I>
where
    I: Iterator<Item = Token> + Send + Sync,
{
    info: FileInfo,
    content: &'s str,
    reader: I,
    index: usize,
}

impl<'s, I> FileReader<'s, I>
where
    I: Iterator<Item = Token> + Send + Sync,
{
    fn into_arc_str(s: String) -> Arc<str> {
        let s = Box::into_raw(s.into_boxed_str());
        unsafe { Arc::from_raw(s) }
    }

    /// Creates a file reader with given file name, content, and reader over tokens
    /// You should ensure that the reader is corresponding to the content, or the behavior may be
    /// unexpected
    pub fn new(
        file: impl Into<String>,
        content: &'s str,
        reader: impl IntoIterator<Item = Token, IntoIter = I>,
    ) -> Self {
        Self {
            info: FileInfo {
                file: Self::into_arc_str(file.into()),
                line: 1,
                column: 1,
            },
            content,
            reader: reader.into_iter(),
            index: 0,
        }
    }

    /// Advance the file reader by one token, returning the next [`ParserInput`](`crate::ParserInput`)
    /// Returns [`None`] when the content or the reader exhausts
    pub fn advance(&mut self) -> Option<crate::ParserInput<'s>> {
        self.reader.next().and_then(|token| {
            let next_index = self.index + token.len;
            let value = self.content.get(self.index..next_index)?;

            self.index = next_index;
            match token.kind {
                TokenKind::Spacing(imuc_lexer::token::Spacing::LineBreak) => {
                    self.info.line += 1;
                    self.info.column = 1;
                }
                _ => {
                    self.info.column += token.len;
                }
            }
            Some(crate::ParserInput {
                kind: token.kind,
                value,
            })
        })
    }
}

impl<'s, I> Iterator for FileReader<'s, I>
where
    I: Iterator<Item = Token> + Send + Sync,
{
    type Item = crate::ParserInput<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.advance() {
            // This filters the unneeded elements for parsing
            if let TokenKind::Comment(_) | TokenKind::Spacing(_) | TokenKind::Stray = item.kind {
            } else {
                #[cfg(debug_assertions)]
                println!("EMIT: {} of kind {:?}", item.value, item.kind);
                return Some(item);
            }
        }
        None
    }
}

impl<'s, I> crate::ParserSequence<'s> for FileReader<'s, I>
where
    I: Iterator<Item = Token> + Send + Sync,
{
    fn map_error(&self, err: Error) -> Error {
        err.context(self.info.clone())
    }
}

impl Display for FileInfo {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(
            f,
            "file {:?} line {} column {}",
            self.file.as_ref(),
            self.line,
            self.column
        )
    }
}
