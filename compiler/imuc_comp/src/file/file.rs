use crate::prelude::*;
use imuc_lexer::{Cursor, Filename};
use std::fs;
use std::sync::OnceLock;

/// Records all information of the file content, providing cursor querying
/// You should use Arc to allow simultaneous access
pub struct FileHandle {
    file: Filename,
    content: OnceLock<FileContent>,
}

impl FileHandle {
    pub fn new(file: Filename) -> Self {
        Self {
            file,
            content: OnceLock::default(),
        }
    }

    fn get_content(&self) -> &FileContent {
        self.content.get_or_init(|| {
            let file = self.file.get();

            debug!("Reading file content from {}", file);

            let content = fs::read_to_string(file.as_str())
                .inspect_err(|err| {
                    error!("Unable to open file {}, {:?}", file, err);
                })
                .unwrap_or_default();
            FileContent::new(content)
        })
    }

    /// Gets the file content as a string, may block on file reads
    pub fn content(&self) -> &str {
        self.get_content().content.as_str()
    }

    /// Converts the cursor into indices in the file string, if the cursor is legal
    pub fn query(&self, cursor: Cursor) -> Option<usize> {
        self.get_content().query(cursor)
    }
}

/// Helper struct of [`FileHandle`] to save file string for querying convenience
#[derive(Default)]
struct FileContent {
    content: String,
    /// Indices of line breaks in the content, note that cursor line numbers start with 1, but this
    /// starts with 0
    lines: Vec<usize>,
}

impl FileContent {
    /// Creates a new file content with the given content
    fn new(content: String) -> Self {
        let mut lines = Vec::new();
        let mut index = 0;
        for ch in content.chars() {
            if ch == '\n' {
                lines.push(index);
            }
            index += ch.len_utf8();
        }
        Self { content, lines }
    }

    /// Queries the location in bytes of the cursor, if any
    fn query(&self, cursor: imuc_lexer::Cursor) -> Option<usize> {
        // Convert start with 1 line numbers to indices that start with 0
        let line = cursor.line.checked_sub(1)?;
        if let Some(start) = self.lines.get(line).copied() {
            let end = self
                .lines
                .get(line + 1)
                .copied()
                .unwrap_or(self.content.len());
            let diff = end.saturating_sub(start);
            if cursor.column > diff {
                None
            } else {
                Some(start + cursor.column)
            }
        } else {
            None
        }
    }
}
