use crate::file::file::*;
use crate::prelude::*;
use imuc_error::errors::ctx::ConvError;
use imuc_lexer::Filename;
use std::collections::{HashMap, hash_map::Entry};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, RwLock};

/// A map from absolute paths to file handles, providing access and pretty error formatting
#[derive(Default)]
pub struct FileMap {
    map: RwLock<HashMap<PathBuf, Arc<FileHandle>>>,
}

fn count_digits(mut value: usize) -> usize {
    let mut result = 0;
    while value > 0 {
        value /= 10;
        result += 1;
    }
    result
}

impl FileMap {
    /// Queries a file name from the map, or creates one if not already created,
    /// returning any file access errors
    pub fn query(&self, filename: Filename) -> Result<Arc<FileHandle>> {
        let path = Path::new(filename.get().as_str()).canonicalize()?;
        let mut map = self.map.write().unwrap();
        match map.entry(path) {
            Entry::Occupied(occupied) => Ok(occupied.get().clone()),
            Entry::Vacant(vacant) => {
                let result = vacant.insert(Arc::new(FileHandle::new(filename))).clone();
                Ok(result)
            }
        }
    }

    #[allow(clippy::single_match)]
    pub fn report_error(&self, err: ConvError) {
        use imuc_error::errors::ctx::*;
        use std::fmt::Write;

        let mut result = String::new();

        match self.query(err.span.file) {
            Ok(handle) => {
                match err.message {
                    Message::Text(ref text) => {
                        result.push_str(text.head.as_str());
                        result.push('\n');
                    }
                    _ => {}
                }

                // NOTE: Unwrap is used because there are no IO errors, and the input should always
                // be valid utf-8
                writeln!(&mut result, "{}", err.span).unwrap();

                let start = handle.query(err.span.start.with_column(1));
                let end = handle.query(err.span.end);
                if let Some((start, end)) = start.and_then(|start| end.map(|end| (start, end))) {
                    let mut current_line = err.span.start.line;
                    let align = count_digits(err.span.end.line);
                    let mut line_break = true;
                    for ch in handle.content()[start..end].chars() {
                        if line_break {
                            let number = current_line.to_string();
                            let mut current_digits = number.len();
                            while current_digits < align {
                                result.push(' ');
                                current_digits += 1;
                            }
                            result.push_str(number.as_str());
                            result.push(' ');
                            result.push('|');
                            result.push(' ');
                            line_break = false;
                        }
                        if ch == '\n' {
                            line_break = true;
                            current_line += 1;
                        }
                        result.push(ch);
                    }
                } else {
                    result.push_str("<missing content>");
                };
                result.push('\n');

                match err.message {
                    Message::Text(ref text) => {
                        if let Some(ref note) = text.note {
                            result.push_str(note);
                            result.push('\n');
                        }
                    }
                    _ => {}
                }
            }
            Err(err) => {
                error!("Error when reporting errors: {err}");
            }
        }

        let level = match err.severity {
            Severity::Note => log::Level::Info,
            Severity::Warn => log::Level::Warn,
            Severity::Error => log::Level::Error,
            Severity::Fatal => log::Level::max(),
        };
        log::log!(level, "{result}");
    }
}

pub static FILE_MAP: LazyLock<FileMap> = LazyLock::new(Default::default);
