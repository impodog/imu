use super::*;
use crate::prelude::*;
use imuc_ast::Span;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    Note,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Error)]
pub struct ConvError {
    pub severity: Severity,
    pub span: Span,
    pub message: Message,
}

#[derive(Debug)]
pub enum Message {
    Text(Text),
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note => write!(f, "Note"),
            Self::Warn => write!(f, "Warn"),
            Self::Error => write!(f, "Error"),
            Self::Fatal => write!(f, "Fatal"),
        }
    }
}

impl fmt::Display for ConvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.sign_plus() {
            // TODO: Add detailed error messages
            todo!("Add detailed error messages with file content pointers");
        } else {
            fmt::Display::fmt(&self.severity, f)?;
            writeln!(f, "{}: {}", self.span, self.message)?;
        }
        Ok(())
    }
}
