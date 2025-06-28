use super::*;
use crate::*;
use imuc_lexer::Span;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    Note,
    Warn,
    #[default]
    Error,
    Fatal,
}

#[derive(Debug, Error, Default)]
pub struct ConvError {
    pub severity: Severity,
    pub span: Span,
    pub message: Message,
}

#[derive(Debug, Default)]
pub enum Message {
    #[default]
    Empty,
    Text(text::Text),
}

impl ConvError {
    /// Creates a conversion error with given severity and error span. The message should be
    /// specified later using 'with' functions
    pub fn new(severity: Severity, span: Span) -> Self {
        Self {
            severity,
            span,
            ..Default::default()
        }
    }
    /// Sets the message of this error to a single head message
    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.message = Message::Text(text::Text {
            head: head.into(),
            note: None,
        });
        self
    }
    /// Sets the message of this error to a text message with both head and note
    pub fn with_text(mut self, head: impl Into<String>, note: impl Into<String>) -> Self {
        self.message = Message::Text(text::Text {
            head: head.into(),
            note: Some(note.into()),
        });
        self
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fatal => write!(f, "Fatal"),
            Self::Error => write!(f, "Error"),
            Self::Warn => write!(f, "Warn"),
            Self::Note => write!(f, "Note"),
        }
    }
}

impl fmt::Display for ConvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", self.severity, self.span)?;
        match &self.message {
            Message::Empty => {
                write!(f, "<error message is to-do>")?;
            }
            Message::Text(text) => {
                write!(f, "{}", text.head)?;
                if let Some(ref note) = text.note {
                    write!(f, "\n\t{note}")?;
                }
            }
        }
        writeln!(f)?;
        Ok(())
    }
}
