use crate::prelude::*;
use ast::expr::BinExpr;

use imuc_ast::expr::Expr;
use imuc_lexer::token::{BinOp, ResTy};
use ir::sym::ty::TyKind;

use super::ExprSolver;

fn resolve_function(ctx: &mut Ctx, head: &Value, nest: &str) -> Option<Value> {
    let item = ctx::mangle::mangle_ty_item(&head.ty.name, nest);
    convs::expr::value::get_glob(ctx, item.as_str())
}

fn resolve_member(
    ctx: &mut Ctx,
    head: &Value,
    nest: &str,
    span: imuc_lexer::Span,
) -> Result<Option<Value>> {
    // NOTE: This function only returns Err because of type resolution, which is not possible in a
    // normal parsing file
    match &head.ty.kind {
        TyKind::Cus(cus) => {
            // FIXME: Anyway to prevent iterating?
            let mut ptr = Ptr::default();
            for (name, ty) in cus.0.iter() {
                if name.as_str() == nest {
                    return Ok(Some(Value {
                        ptr: head.ptr + ptr,
                        ty: ctx
                            .ty
                            .resolve_or(ty, span)
                            .map_err(|err| {
                                ctx.push_error(err);
                                SendError::new_error()
                            })?
                            .clone(),
                    }));
                }
                ptr += ty.size().ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(Severity::Fatal, span)
                            .with_head("Resolved type required in member resolution"),
                    );
                    SendError::new_error()
                })?;
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[allow(clippy::collapsible_else_if)]
fn resolve_nest(
    ctx: &mut Ctx,
    mut head: Value,
    nest: &str,
    span: imuc_lexer::Span,
) -> Result<Value> {
    let is_self = ctx
        .body()
        .self_ty()
        .is_some_and(|self_ty| self_ty.test_eq(&head.ty));
    let resolved = if is_self {
        if let Some(value) = resolve_member(ctx, &head, nest, span)? {
            head = value;
            true
        } else if let Some(value) = resolve_function(ctx, &head, nest) {
            head = value;
            true
        } else {
            false
        }
    } else {
        if let Some(value) = resolve_function(ctx, &head, nest) {
            head = value;
            true
        } else if let Some(value) = resolve_member(ctx, &head, nest, span)? {
            head = value;
            true
        } else {
            false
        }
    };
    if !resolved {
        ctx.push_error(ConvError::new(Severity::Error, span).with_text(
            "Unable to resolve member or function",
            format!("Unable to resolve {} of type {}", nest, head.ty.name),
        ));
        return Err(SendError::new_error());
    }
    Ok(head)
}

pub struct BinExprConv {
    pub solver: ExprSolver,
}

impl Converter for BinExprConv {
    type Input = BinExpr;
}

enum BinOpKind {
    Arithmetic(fn(NumBytes, Bytes, Ptr) -> Cmd, Bytes),
    Compare(i8),
    CompareNot(i8),
}

impl Convert<Value> for BinExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        // TODO: Hint the type with values solved
        let Self { solver } = self;

        // Special case: Call operator
        if input.op == BinOp::Call {
            return crate::convs::expr::call::convert_call(
                ctx,
                &input.rhs,
                &input.lhs,
                input.span(),
            );
        }

        let lhs: Value = convs::ExprConv {
            solver: ExprSolver::inherit(&solver),
        }
        .convert(ctx, input.lhs.as_ref())?
        .ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span).with_head("Lhs should return a value"),
            );
            SendError::default()
        })?;

        // Special case: Dot is not an arithmetic operator
        if input.op == BinOp::Dot {
            match input.rhs.as_ref() {
                Expr::Value(ast::expr::Value {
                    value: ast::expr::ValueInner::Name(name),
                    ..
                }) => {
                    if let Some(self_value) = solver.self_value {
                        self_value.get_or_init(|| lhs.clone());
                    }
                    return resolve_nest(ctx, lhs, name.as_str(), input.span);
                }
                _ => {
                    ctx.push_error(
                        ConvError::new(Severity::Error, input.span())
                            .with_head("Value name required after dot operator"),
                    );
                    return Err(SendError::new_error());
                }
            }
        }

        let rhs: Value = convs::ExprConv {
            solver: ExprSolver::inherit(&solver).with_hint_or_else(Some(lhs.ty.clone())),
        }
        .convert(ctx, input.rhs.as_ref())?
        .ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span).with_head("Rhs should return a value"),
            );
            SendError::default()
        })?;
        let lhs_ty = lhs.ty.to_res_ty().ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span)
                    .with_head("BinOp can only be applied to primitive lhs"),
            );
            SendError::default()
        })?;
        let rhs_ty = rhs.ty.to_res_ty().ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span)
                    .with_head("BinOp can only be applied to primitive rhs"),
            );
            SendError::default()
        })?;
        if lhs_ty != rhs_ty {
            ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                "BinOp requires two values of the same type",
                format!("Two types are {} and {}", lhs.ty.name, rhs.ty.name),
            ));
            return Err(SendError::default().into());
        }
        let is_float = matches!(lhs_ty, ResTy::F32 | ResTy::F64);
        let bytes: NumBytes = lhs_ty.try_into()?;
        let opd_bytes = lhs.ty.size_or(input.span).map_err(ctx.push_error_fn())?;

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
            BinOp::Ne => BinOpKind::CompareNot(0),
            BinOp::Lt => BinOpKind::Compare(-1),
            BinOp::Gt => BinOpKind::Compare(1),
            BinOp::Le => BinOpKind::CompareNot(1),
            BinOp::Ge => BinOpKind::CompareNot(-1),
            BinOp::Dot => unreachable!("Dot operator is filtered"),
            BinOp::Call => unreachable!("Call operator is filtered"),
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
                let compare_ptr = body.push_stack(Bytes::byte());
                let ptr = body.push_stack(Bytes::byte());
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
            BinOpKind::CompareNot(target) => {
                let body = ctx.body_mut();
                let compare_ptr = body.push_stack(Bytes::byte());
                let inverse_ptr = body.push_stack(Bytes::byte());
                let ptr = body.push_stack(Bytes::byte());
                if is_float {
                    body.push(Cmd::Testf(bytes, lhs.ptr, rhs.ptr));
                } else {
                    body.push(Cmd::Test(bytes, lhs.ptr, rhs.ptr));
                }
                body.push(Cmd::EqI8(compare_ptr, target));
                body.push(Cmd::Not(NumBytes::I8, inverse_ptr));
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
        }
    }
}
