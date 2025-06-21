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
            Self::Not | Self::Ref | Self::Ptr => 5,
        }
    }

    fn is_right(&self) -> bool {
        true
    }
}

impl Priority for BinOp {
    fn priority(&self) -> u8 {
        match self {
            Self::Dot => 1,
            Self::Call => 2,
            Self::Mul | Self::Div => 7,
            Self::Add | Self::Sub => 8,
            Self::Eq | Self::Ne | Self::Lt | Self::Le | Self::Gt | Self::Ge => 13,
            Self::And => 14,
            Self::Xor => 15,
            Self::Or => 16,
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
