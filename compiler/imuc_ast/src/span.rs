use imuc_lexer::StrRef;

#[derive(Debug, Clone)]
pub struct Span {
    pub file: StrRef,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

// FIXME: Add spans for more structures
