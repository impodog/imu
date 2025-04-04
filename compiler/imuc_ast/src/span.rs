use imuc_lexer::Filename;

#[derive(Debug, Clone, Copy)]
/// A span containing file name, cursor position(to the *rightmost* character, not inclusive), and covered length,
/// useful for error reporting
///
/// The rules for adding a span to an AST node is that when the node is specific enough to create a
/// relevant error on its own, then a span field is added
pub struct Span {
    pub file: Filename,
    pub line: usize,
    pub column: usize,
    pub len: usize,
}

impl Span {
    /// Creates a referenced span that has no specific file location
    ///
    /// This is used when generating mid-process AST nodes
    pub fn refer(len: usize) -> Self {
        Self {
            file: Default::default(),
            line: 0,
            column: 0,
            len,
        }
    }
}
