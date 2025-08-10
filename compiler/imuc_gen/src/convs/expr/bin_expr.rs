use crate::prelude::*;
use ast::expr::BinExpr;

use imuc_ast::expr::Expr;
use imuc_ir::sym::ty::{TyInner, TyItem};
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
            for (name, field) in cus.0.iter() {
                if name.as_str() == nest {
                    return Ok(Some(Value {
                        ptr: head.ptr + field.pad,
                        ty: ctx
                            .ty
                            .resolve_or(&field.item, span)
                            .map_err(ctx.push_error_fn())?
                            .clone(),
                    }));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

fn resolve_arrow_member(
    ctx: &mut Ctx,
    head: &Value,
    nest: &str,
    span: imuc_lexer::Span,
) -> Result<Value> {
    // NOTE: This function only returns Err because of type resolution, which is not possible in a
    // normal parsing file
    match &head.ty.kind {
        TyKind::Ptr(item) | TyKind::Ref(item) => {
            let push_error_fn = ctx.push_error_fn();
            let ty = ctx.ty.resolve_or(item, span).map_err(&push_error_fn)?;
            match &ty.kind {
                TyKind::Cus(cus) => {
                    // FIXME: Anyway to prevent iterating?
                    let mut found = None;
                    for (name, field) in cus.0.iter() {
                        if name.as_str() == nest {
                            let ty = ctx
                                .ty
                                .resolve_or(&field.item, span)
                                .map_err(&push_error_fn)?;
                            found = Some((field.pad, ty.clone()));
                            break;
                        }
                    }
                    if let Some((pad, ty)) = found {
                        let ty = if matches!(head.ty.kind, TyKind::Ptr(_)) {
                            let name = StrRef::from(ctx::mangle::mangle_ptr(&ty.name));
                            ctx.ty
                                .or_insert_with(name.clone(), move || {
                                    Ty::new(TyInner::new_priv(name, TyKind::Ptr(TyItem::Solid(ty))))
                                })
                                .clone()
                        } else {
                            let name = StrRef::from(ctx::mangle::mangle_ref(&ty.name));
                            ctx.ty
                                .or_insert_with(name.clone(), move || {
                                    Ty::new(TyInner::new_priv(name, TyKind::Ref(TyItem::Solid(ty))))
                                })
                                .clone()
                        };
                        let body = ctx.body_mut();
                        let operand_ptr = body.push_cmd(
                            Bytes::ptr(),
                            Cmd::Store(ast::prim::Prim::Integer(ast::prim::Integer::ptr(
                                pad.num(),
                            ))),
                        );
                        let value_ptr = body.push_cmd(
                            Bytes::ptr(),
                            Cmd::Add(ir::cmd::PTR_BYTES, head.ptr, operand_ptr),
                        );
                        Ok(Value { ptr: value_ptr, ty })
                    } else {
                        Err(ConvError::new(Severity::Error, span)
                            .with_text(
                                "Unable to find field",
                                format!("No field {} in type {}", nest, ty.name),
                            )
                            .into())
                    }
                }
                _ => Err(ConvError::new(Severity::Error, span)
                    .with_text(
                        "Expected pointer or reference to Cus",
                        format!("Found {}", head.ty.name),
                    )
                    .into()),
            }
        }
        _ => Err(ConvError::new(Severity::Error, span)
            .with_text(
                "Expected pointer or reference",
                format!("Found {}", head.ty.name),
            )
            .into()),
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

fn convert_back_arrow(
    ctx: &mut Ctx,
    lhs: Value,
    rhs: Value,
    span: imuc_lexer::Span,
) -> Result<Value> {
    match &lhs.ty.kind {
        TyKind::Ref(ty) => {
            let push_error_fn = ctx.push_error_fn();
            let ty = ctx.ty.resolve_or(ty, span).map_err(&push_error_fn)?;
            let size = ty.size_or(span).map_err(&push_error_fn)?;
            if ty.test_eq(&rhs.ty) {
                let body = ctx.body_mut();
                body.push_void(Cmd::WriteHeap(size, Bytes::start(), rhs.ptr, lhs.ptr));
                Ok(Value::default())
            } else {
                ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                    "Assignment types mismatch",
                    format!("Lhs(Ref) is {}, Rhs is {}", ty.name, rhs.ty.name),
                ));
                Err(SendError::new_error())
            }
        }
        _ => {
            ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                "BackArrow requires Ref on lhs",
                format!("Unable to assign to type {}", lhs.ty.name),
            ));
            Err(SendError::new_error())
        }
    }
}

pub struct BinExprConv {
    pub solver: ExprSolver,
}

impl Converter for BinExprConv {
    type Input = BinExpr;
}

enum BinOpKind {
    Arithmetic(fn(NumBytes, Bytes, Ptr) -> Cmd, Bytes),
    // First is the compare sign(-1, 0, 1), second is whether to filp compare results
    Compare(i8, bool),
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

        // Special case: Dot
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
                            .with_head("Value name required after Dot operator"),
                    );
                    return Err(SendError::new_error());
                }
            }
        }

        // Special case: Arrow
        if input.op == BinOp::Arrow {
            match input.rhs.as_ref() {
                Expr::Value(ast::expr::Value {
                    value: ast::expr::ValueInner::Name(name),
                    ..
                }) => {
                    return resolve_arrow_member(ctx, &lhs, name.as_str(), input.span);
                }
                _ => {
                    ctx.push_error(
                        ConvError::new(Severity::Error, input.span())
                            .with_head("Value name required after Arrow operator"),
                    );
                    return Err(SendError::new_error());
                }
            }
        }

        // Determines the hint type to use
        let hint_ty = if input.op == BinOp::BackArrow {
            match &lhs.ty.kind {
                TyKind::Ref(item) => {
                    if let Some(ty) = ctx.ty.resolve(item) {
                        ty.clone()
                    } else {
                        lhs.ty.clone()
                    }
                }
                _ => lhs.ty.clone(),
            }
        } else {
            lhs.ty.clone()
        };

        let rhs: Value = convs::ExprConv {
            solver: ExprSolver::inherit(&solver).with_hint_or_else(Some(hint_ty)),
        }
        .convert(ctx, input.rhs.as_ref())?
        .ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span).with_head("Rhs should return a value"),
            );
            SendError::default()
        })?;

        // Special case: BackArrow assignment
        if input.op == BinOp::BackArrow {
            return convert_back_arrow(ctx, lhs, rhs, input.span);
        }

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
            BinOp::Eq => BinOpKind::Compare(0, false),
            BinOp::Ne => BinOpKind::Compare(0, true),
            BinOp::Lt => BinOpKind::Compare(-1, false),
            BinOp::Gt => BinOpKind::Compare(1, false),
            BinOp::Le => BinOpKind::Compare(1, true),
            BinOp::Ge => BinOpKind::Compare(-1, true),
            BinOp::Dot => unreachable!("Dot operator is filtered"),
            BinOp::Call => unreachable!("Call operator is filtered"),
            BinOp::Arrow => unreachable!("Arrow operator is filtered"),
            BinOp::BackArrow => unreachable!("BackArrow operator is filtered"),
        };

        match kind {
            BinOpKind::Arithmetic(func, size) => {
                let body = ctx.body_mut();
                let ptr = body.push_cmd(size, func(bytes, lhs.ptr, rhs.ptr));
                Ok(Value {
                    ptr,
                    ty: lhs.ty.clone(),
                })
            }
            BinOpKind::Compare(target, flip) => {
                let body = ctx.body_mut();
                let compare_ptr = body.push_cmd(
                    Bytes::byte(),
                    if is_float {
                        Cmd::Testf(bytes, lhs.ptr, rhs.ptr)
                    } else {
                        Cmd::Test(bytes, lhs.ptr, rhs.ptr)
                    },
                );
                let ptr = body.push_cmd(
                    Bytes::byte(),
                    if flip {
                        Cmd::EqI8(compare_ptr, target)
                    } else {
                        Cmd::NeI8(compare_ptr, target)
                    },
                );
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
        }
    }
}
