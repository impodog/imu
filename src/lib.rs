#[cfg(feature = "compiler")]
mod compiler {
    pub use imuc_lexer::Reader;
    pub use imuc_parser::{FileReader, Parser, Rule};
    pub use imuc_rules::rules;
}

#[cfg(feature = "compiler")]
pub use crate::compiler::*;
