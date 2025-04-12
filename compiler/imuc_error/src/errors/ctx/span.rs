use imuc_lexer::Filename;
use std::fmt;

#[derive(Debug, Clone, Copy, Default)]
/// A cursor line-column position
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, Default)]
/// A span containing file name, cursor position(left inclusive, right non-inclusive),
/// useful for error reporting
///
/// The rules for adding a span to an AST node is that when the node is specific enough to create a
/// relevant error on its own, then a span field is added
pub struct Span {
    pub file: Filename,
    pub start: Cursor,
    pub end: Cursor,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.sign_plus() {
            // FIXME: Read the file content, seek the cursor location
            todo!()
        } else {
            write!(
                f,
                "{}@[{}:{}~{}:{}]",
                self.file.get(),
                self.start.line,
                self.start.column,
                self.end.line,
                self.end.column
            )?;
        }
        Ok(())
    }
}

impl Span {
    /// Creates a referenced span that has no specific file location
    ///
    /// This is used when generating mid-process AST nodes
    pub fn refer() -> Self {
        Self {
            file: Default::default(),
            ..Default::default()
        }
    }
}
