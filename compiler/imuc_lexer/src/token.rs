/// A enum representing the kind only, used to assemble the actual token
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenKind {
    Comment(Comment),
    Spacing(Spacing),
    Literal(Literal),
    Ident(Ident),
    Keyword(Keyword),
    ResTy(ResTy),
    ResVal(ResVal),
    Pair(Pair),
    BinOp(BinOp),
    UnOp(UnOp),
    Symbol(Symbol),
    LexError(LexError),

    // Special file structure tokens
    Semicolon,
    Stray,
    Eof,
}

/// A part of [`TokenKind`] for comments
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Comment {
    Comment,
    MultiComment,
}

/// A part of [`TokenKind`] for chunks of whitespace
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Spacing {
    Indent,
    LineBreak,
}

/// A part of [`TokenKind`] for literals
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Literal {
    Integer,
    Float,
    String,
    MultiString,
}

/// A part of [`TokenKind`] for identifiers(values / types / ignore names)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Ident {
    Value,
    Type,
    Unused,
}

/// A part of [`TokenKind`] for keywords
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Keyword {
    Pub,
    Mut,
    Fun,
    Cus,
    Val,
    For,
    As,
    If,
    Else,
    Loop,
}

/// A part of [`TokenKind`] for values using reserved names
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResVal {
    True,
    False,
    SelfValue,
}

/// A part of [`TokenKind`] for types, such as primitives, using reserved names
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResTy {
    SelfType,
    I8,
    I16,
    I32,
    I64,
    I128,
    Ptr,
    F32,
    F64,
    Str,
}

/// A part of [`TokenKind`] for left / right bracket
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Pair {
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
}

/// A part of [`TokenKind`] for binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinOp {
    Dot,
    Add,
    Sub,
    Mul,
    Div,
    Or,
    And,
    Xor,
    Assign,
}

/// A part of [`TokenKind`] for unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnOp {
    Neg,
}

/// A part of [`TokenKind`] for parser structure symbol
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Symbol {
    Colon,
    Comma,
}

/// A part of [`TokenKind`] for errors that may happen in lexer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LexError {
    UnknownChar,
    NumberError,
    UnclosedComment,
    UnclosedString,
}

/// The token produced by the lexer of [`crate::reader::Reader`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// The length in bytes of the token
    /// The start position can be calculated from the previous tokens
    pub len: usize,
}

impl Token {
    pub fn new(kind: TokenKind, len: usize) -> Token {
        Token { kind, len }
    }
}
