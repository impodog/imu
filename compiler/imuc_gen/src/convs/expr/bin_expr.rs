use crate::prelude::*;
use ast::expr::BinExpr;
use ctx::Value;
use imuc_lexer::token::BinOp;

pub struct BinExprConv;

impl Converter for BinExprConv {
    type Input = BinExpr;
}

enum BinOpKind {
    Arithmetic(fn(NumBytes, Bytes, Ptr) -> Cmd),
    Compare(i8),
    CompareEq(i8),
}

impl Convert<Option<Value>> for BinExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        let lhs: Value = convs::ExprConv
            .convert(ctx, input.lhs.as_ref())?
            .ok_or_else(|| ctx.map_err(errors::ConvError::ValueRequired("BinExpr".to_owned())))?;
        let rhs: Value = convs::ExprConv
            .convert(ctx, input.rhs.as_ref())?
            .ok_or_else(|| ctx.map_err(errors::ConvError::ValueRequired("BinExpr".to_owned())))?;

        let kind = match input.op {
            BinOp::Add => BinOpKind::Arithmetic(Cmd::Add),
            BinOp::Sub => BinOpKind::Arithmetic(Cmd::Sub),
            BinOp::Mul => BinOpKind::Arithmetic(Cmd::Mul),
            BinOp::Div => BinOpKind::Arithmetic(Cmd::Div),
            BinOp::Or => BinOpKind::Arithmetic(Cmd::Or),
            BinOp::And => BinOpKind::Arithmetic(Cmd::And),
            BinOp::Xor => BinOpKind::Arithmetic(Cmd::Xor),
            BinOp::Eq => BinOpKind::Arithmetic(Cmd::Eq),
            BinOp::Lt => BinOpKind::Compare(-1),
            BinOp::Gt => BinOpKind::Compare(1),
            BinOp::Le => BinOpKind::CompareEq(-1),
            BinOp::Ge => BinOpKind::CompareEq(1),
        };
        todo!()
    }
}
