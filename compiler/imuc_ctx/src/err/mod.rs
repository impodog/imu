use crate::prelude::*;

#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Fatal,
    Error,
    Warn,
    Note,
}
