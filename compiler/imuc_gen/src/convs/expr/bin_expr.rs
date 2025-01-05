use crate::prelude::*;
use ast::expr::BinExpr;

use imuc_lexer::token::{BinOp, ResTy};

pub struct BinExprConv;

impl Converter for BinExprConv {
    type Input = BinExpr;
}

enum BinOpKind {
    Arithmetic(fn(NumBytes, Bytes, Ptr) -> Cmd, Bytes),
    Compare(i8),
    CompareEq(i8),
}

impl Convert<Value> for BinExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let lhs: Value = convs::ExprConv
            .convert(ctx, input.lhs.as_ref())?
            .ok_or_else(|| errors::ConvError::ValueRequired("BinExpr".to_owned()))?;
        let rhs: Value = convs::ExprConv
            .convert(ctx, input.rhs.as_ref())?
            .ok_or_else(|| errors::ConvError::ValueRequired("BinExpr".to_owned()))?;
        let lhs_ty = lhs
            .ty
            .to_res_ty()
            .ok_or_else(|| errors::ConvError::PrimitiveRequired("BinExpr".to_owned()))?;
        let rhs_ty = rhs
            .ty
            .to_res_ty()
            .ok_or_else(|| errors::ConvError::PrimitiveRequired("BinExpr".to_owned()))?;
        if lhs_ty != rhs_ty {
            return Err(errors::ConvError::TypesMismatch(format!(
                "lhs: {:?}, rhs: {:?}",
                lhs_ty, rhs_ty
            ))
            .into());
        }
        let is_float = matches!(lhs_ty, ResTy::F32 | ResTy::F64);
        let bytes: NumBytes = lhs_ty.try_into()?;
        let opd_bytes = lhs.ty.size_or()?;

        let kind = match input.op {
            BinOp::Add => {
                BinOpKind::Arithmetic(if is_float { Cmd::Addf } else { Cmd::Add }, opd_bytes)
            }
            BinOp::Sub => {
                BinOpKind::Arithmetic(if is_float { Cmd::Subf } else { Cmd::Sub }, opd_bytes)
            }
            BinOp::Mul => {
                BinOpKind::Arithmetic(if is_float { Cmd::Mulf } else { Cmd::Mul }, opd_bytes)
            }
            BinOp::Div => {
                BinOpKind::Arithmetic(if is_float { Cmd::Divf } else { Cmd::Div }, opd_bytes)
            }
            BinOp::Or => BinOpKind::Arithmetic(Cmd::Or, opd_bytes),
            BinOp::And => BinOpKind::Arithmetic(Cmd::And, opd_bytes),
            BinOp::Xor => BinOpKind::Arithmetic(Cmd::Xor, opd_bytes),
            BinOp::Eq => BinOpKind::Compare(0),
            BinOp::Lt => BinOpKind::Compare(-1),
            BinOp::Gt => BinOpKind::Compare(1),
            BinOp::Le => BinOpKind::CompareEq(-1),
            BinOp::Ge => BinOpKind::CompareEq(1),
        };

        match kind {
            BinOpKind::Arithmetic(func, size) => {
                let body = ctx.body_mut();
                let ptr = body.push_stack(size);
                body.push(func(bytes, lhs.ptr, rhs.ptr));
                Ok(Value {
                    ptr,
                    ty: lhs.ty.clone(),
                })
            }
            BinOpKind::Compare(target) => {
                let body = ctx.body_mut();
                let compare_ptr = body.push_stack(Bytes::new(1));
                let ptr = body.push_stack(Bytes::new(1));
                if is_float {
                    body.push(Cmd::Testf(bytes, lhs.ptr, rhs.ptr));
                } else {
                    body.push(Cmd::Test(bytes, lhs.ptr, rhs.ptr));
                }
                body.push(Cmd::EqI8(compare_ptr, target));
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
            BinOpKind::CompareEq(target) => {
                let body = ctx.body_mut();
                let compare_ptr = body.push_stack(Bytes::new(1));
                let compare_lhs = body.push_stack(Bytes::new(1));
                let compare_rhs = body.push_stack(Bytes::new(1));
                let ptr = body.push_stack(Bytes::new(1));
                if is_float {
                    body.push(Cmd::Testf(bytes, lhs.ptr, rhs.ptr));
                } else {
                    body.push(Cmd::Test(bytes, lhs.ptr, rhs.ptr));
                }
                body.push(Cmd::EqI8(compare_ptr, target));
                body.push(Cmd::EqI8(compare_ptr, 0));
                body.push(Cmd::Or(NumBytes::I8, compare_lhs, compare_rhs));
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
        }
    }
}
