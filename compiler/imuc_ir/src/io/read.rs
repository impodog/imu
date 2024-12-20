use crate::prelude::*;
use std::io::BufRead;

/// Helper trait for UTF-8 reading used in IR serde
pub trait IrRead {
    /// Reads the next char, regarding EOF as an error
    fn read_char(&mut self) -> Result<char>;
    /// Reads until the reader hits [`ch`] or reaches the end of a line, consuming the "until" character
    fn read_until(&mut self, ch: char) -> Result<&str>;
    /// Reads until the reader reaches the end of a line, consuming the rest of the line
    fn read_line(&mut self) -> Result<&str>;
    /// Peeks the current line without consuming any characters, returns None when EOF is reached
    fn peek_line(&mut self) -> Option<&str>;
    /// Peeks the current line, returning Err on EOF
    ///
    /// The default implementation uses [`Self::peek_line`]
    fn peek_line_or_else(&mut self) -> Result<&str> {
        self.peek_line().ok_or_else(|| errors::IrError::Eof.into())
    }
    /// Returns whether the reading is outside the current compiled module
    fn external(&self) -> bool;
}

impl<T> IrRead for &mut T
where
    T: IrRead,
{
    fn read_char(&mut self) -> Result<char> {
        (**self).read_char()
    }
    fn read_until(&mut self, ch: char) -> Result<&str> {
        (**self).read_until(ch)
    }
    fn read_line(&mut self) -> Result<&str> {
        (**self).read_line()
    }
    fn peek_line(&mut self) -> Option<&str> {
        (**self).peek_line()
    }
    fn external(&self) -> bool {
        (**self).external()
    }
}

/// This wrapper implements [`IrRead`] and can be used for IR serde
pub struct IrReader<T>
where
    T: BufRead,
{
    inner: T,
    line: Option<String>,
    cursor: usize,
    external: bool,
}

impl<T> IrReader<T>
where
    T: BufRead,
{
    /// Creates a new [`IrReader`]
    pub fn new(inner: T, external: bool) -> Self {
        Self {
            inner,
            line: None,
            cursor: 0,
            external,
        }
    }

    /// Updates to the next non-empty line, if the previous one is done reading
    ///
    /// Returns Ok(false) when EOF is hit
    pub fn update(&mut self) -> Result<bool> {
        while self
            .line
            .as_ref()
            .is_none_or(|line| self.cursor >= line.len())
        {
            let mut buf = String::new();
            let value = self.inner.read_line(&mut buf)?;
            buf.pop().expect("a line contains at least one character");
            self.line = Some(buf);
            if value == 0 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl<T> IrRead for IrReader<T>
where
    T: BufRead,
{
    fn read_char(&mut self) -> Result<char> {
        if !self.update()? {
            return Err(errors::IrError::Eof.into());
        }
        if let Some(line) = &self.line {
            let ch = line[self.cursor..]
                .chars()
                .next()
                .expect("chars should not be empty");
            self.cursor += ch.len_utf8();
            Ok(ch)
        } else {
            unreachable!()
        }
    }
    fn read_until(&mut self, until: char) -> Result<&str> {
        if !self.update()? {
            return Err(errors::IrError::Eof.into());
        }
        let begin = self.cursor;
        if let Some(line) = &mut self.line {
            while self.cursor < line.len() {
                let ch = line[self.cursor..]
                    .chars()
                    .next()
                    .expect("chars should not be empty");
                self.cursor += ch.len_utf8();
                if ch == until {
                    break;
                }
            }
            Ok(&line[begin..self.cursor])
        } else {
            unreachable!()
        }
    }

    fn read_line(&mut self) -> Result<&str> {
        if !self.update()? {
            return Err(errors::IrError::Eof.into());
        }
        if let Some(line) = &self.line {
            let result = &line[self.cursor..];
            // Setting cursor to usize::MAX effectively forces [`Self::update`] on next read
            self.cursor = usize::MAX;
            Ok(result)
        } else {
            unreachable!()
        }
    }

    fn peek_line(&mut self) -> Option<&str> {
        if !self.update().ok()? {
            return None;
        }
        self.line.as_deref()
    }

    fn external(&self) -> bool {
        self.external
    }
}

/// Wrapper around a string reference without '\n' character
pub struct LineReader<'s> {
    value: &'s str,
    cursor: usize,
    external: bool,
}

impl<'s> LineReader<'s> {
    pub fn new(value: &'s str, external: bool) -> Self {
        Self {
            value,
            cursor: 0,
            external,
        }
    }
}

impl IrRead for LineReader<'_> {
    fn read_char(&mut self) -> Result<char> {
        if self.cursor < self.value.len() {
            let ch = self.value[self.cursor..]
                .chars()
                .next()
                .expect("chars should not be empty");
            self.cursor += ch.len_utf8();
            Ok(ch)
        } else {
            Err(errors::IrError::Eof.into())
        }
    }

    fn read_until(&mut self, until: char) -> Result<&str> {
        let begin = self.cursor;
        while self.cursor < self.value.len() {
            let ch = self.value[self.cursor..]
                .chars()
                .next()
                .expect("chars should not be empty");
            self.cursor += ch.len_utf8();
            if ch == until {
                break;
            }
        }
        if begin == self.cursor {
            Err(errors::IrError::Eof.into())
        } else {
            Ok(&self.value[begin..self.cursor])
        }
    }

    fn read_line(&mut self) -> Result<&str> {
        if self.cursor < self.value.len() {
            let result = &self.value[self.cursor..];
            // The line is totally consumed
            self.cursor = usize::MAX;
            Ok(result)
        } else {
            Err(errors::IrError::Eof.into())
        }
    }

    fn peek_line(&mut self) -> Option<&str> {
        if self.cursor < self.value.len() {
            Some(self.value)
        } else {
            None
        }
    }

    fn external(&self) -> bool {
        self.external
    }
}
