use imuc_lexer::token::{BinOp, TokenKind, UnOp};

/// Defines the binding priority for unary and binary operators
///
/// The smaller the priority unmber is, the earlier it will bind operands
pub trait Priority {
    fn priority(&self) -> u8;

    fn is_right(&self) -> bool;
}

impl Priority for UnOp {
    fn priority(&self) -> u8 {
        match self {
            Self::Ref => 5,
        }
    }

    fn is_right(&self) -> bool {
        true
    }
}

impl Priority for BinOp {
    fn priority(&self) -> u8 {
        match self {
            Self::Mul | Self::Div | Self::Mod => 7,
            Self::Add | Self::Sub => 8,
            Self::And => 10,
            Self::Xor => 11,
            Self::Or => 12,
            Self::Eq | Self::Lt | Self::Le | Self::Gt | Self::Ge => 13,
        }
    }

    fn is_right(&self) -> bool {
        false
    }
}

impl Priority for TokenKind {
    fn priority(&self) -> u8 {
        match self {
            Self::BinOp(op) => op.priority(),
            Self::UnOp(op) => op.priority(),
            _ => u8::MAX,
        }
    }

    fn is_right(&self) -> bool {
        match self {
            Self::BinOp(op) => op.is_right(),
            Self::UnOp(op) => op.is_right(),
            _ => false,
        }
    }
}
