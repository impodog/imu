use super::*;
use crate::*;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Fatal,
    Error,
    Warn,
    Note,
}

#[derive(Debug, Error)]
pub struct ConvError {
    pub severity: Severity,
    pub span: Span,
    pub message: Message,
}

#[derive(Debug)]
pub enum Message {
    Text(text::Text),
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
            Message::Text(text) => {
                write!(f, "{}", text.head)?;
                if let Some(ref note) = text.note {
                    write!(f, "\n\t{}", note)?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}
