// use super::ExprSolver;
use crate::prelude::*;
use ast::expr::{BinExpr, Expr};
use imuc_ir::sym::ty::TyItem;
use imuc_lexer::{
    token::{BinOp, ResTy},
    Span,
};
use ir::sym::ty::{TyInner, TyKind};
use std::sync::{Arc, OnceLock};

use super::ExprSolver;

/// NOTE: The error here should be added to the ctx instead of returned
fn add_self_to_args(
    ctx: &mut Ctx,
    args: Value,
    self_value: Value,
    span: Span,
) -> Result<Value, ConvError> {
    let body = ctx.body_mut();

    let self_value_size = self_value.ty.size_or(span)?;
    let args_size = args.ty.size_or(span)?;

    let ptr = body.push_stack(self_value_size + args_size);
    body.push(Cmd::Dupli(self_value.ptr, self_value_size));
    body.push(Cmd::Dupli(args.ptr, args_size));

    use std::iter::once;
    let ty = match &args.ty.kind {
        TyKind::Tuple(tuple) => {
            let name = once('(')
                .chain(self_value.ty.name.chars())
                .chain(once(','))
                .chain(args.ty.name.chars().skip(1))
                .collect::<String>();
            let name = StrRef::from(name);
            ctx.ty
                .or_insert_with(name.clone(), || {
                    let tuple = once(TyItem::Solid(args.ty.clone()))
                        .chain(tuple.0.iter().cloned())
                        .collect::<Vec<_>>();
                    Ty::new(TyInner {
                        name,
                        kind: TyKind::Tuple(ir::sym::ty::Tuple(tuple)),
                        external: true,
                    })
                })
                .clone()
        }
        TyKind::Res(ResTy::Unit) => self_value.ty.clone(),
        _ => {
            let name = once('(')
                .chain(self_value.ty.name.chars())
                .chain(once(','))
                .chain(args.ty.name.chars())
                .chain(once(')'))
                .collect::<String>();
            let name = StrRef::from(name);
            ctx.ty
                .or_insert_with(name.clone(), || {
                    let tuple = [&self_value.ty, &args.ty]
                        .into_iter()
                        .map(|ty| TyItem::Solid(ty.clone()))
                        .collect::<Vec<_>>();
                    Ty::new(TyInner {
                        name,
                        kind: TyKind::Tuple(ir::sym::ty::Tuple(tuple)),
                        external: true,
                    })
                })
                .clone()
        }
    };
    Ok(Value { ptr, ty })
}

pub(crate) fn convert_call(ctx: &mut Ctx, args: &Expr, func: &Expr, span: Span) -> Result<Value> {
    let (self_value, func) = {
        let mut solver = ExprSolver::default();
        // Obtains self value if the function is received from a dot operation
        let self_value = if matches!(func, Expr::BinExpr(BinExpr { op: BinOp::Dot, .. })) {
            let self_value = Arc::new(OnceLock::new());
            solver.self_value = Some(self_value.clone());
            Some(self_value)
        } else {
            None
        };
        let func = convs::ExprConv { solver }
            .convert(ctx, func)?
            .ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, func.unwrap_span())
                        .with_head("Required fun pointer for function call"),
                );
                SendError::new_error()
            })?;
        (self_value, func)
    };
    let args = convs::ExprConv::default()
        .convert(ctx, args)?
        .ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, args.unwrap_span())
                    .with_head("Required arguments for function call"),
            );
            SendError::new_error()
        })?;
    // FIXME: self_value should always exist in the OnceLock since a dot operator is detected
    let args = if let Some(self_value) = self_value.and_then(|self_value| self_value.get().cloned())
    {
        add_self_to_args(ctx, args, self_value, span).map_err(|err| {
            ctx.push_error(err);
            SendError::new_error()
        })?
    } else {
        args
    };

    let ret = match &func.ty.kind {
        TyKind::Ptr(item) => {
            let ty = ctx.ty.resolve_or(item, span)?.clone();
            match &ty.kind {
                TyKind::Fun { param, ret } => {
                    let param = ctx.ty.resolve_or(param, span)?.clone();
                    let ret = ctx.ty.resolve_or(ret, span)?.clone();
                    if !args.ty.test_eq(&param) {
                        ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                            "Function parameter and argument types mismatch",
                            format!("Expected {}, given {}", param.name, args.ty.name),
                        ));
                        return Err(SendError::new_error());
                    }
                    ret
                }
                _ => {
                    ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                        "Expected function to be called",
                        format!("Given value type is {}", func.ty.name),
                    ));
                    return Err(SendError::new_error());
                }
            }
        }
        _ => {
            ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                "Expected function pointer when calling",
                format!("Given value type is {}", func.ty.name),
            ));
            if func.ty.test_eq(&Ty::unit()) {
                ctx.push_error(
                    ConvError::new(Severity::Note, span)
                        .with_head("Did you forget to put a semicolon?"),
                );
            }
            return Err(SendError::new_error());
        }
    };
    // NOTE: This data copy is required regardless of whether the copy is needed, to make sure the
    // data stays in place of a function call, and will be optimized later
    let body = ctx.body_mut();
    // For function pointer
    body.push_stack(Bytes::ptr());
    // For arguments
    let args_size = args.ty.size_or(span)?;
    body.push_stack(args_size);
    // For return value
    let ret_ptr = body.push_stack(ret.size_or(span)?);
    body.push(Cmd::Dupli(Bytes::ptr(), func.ptr));
    body.push(Cmd::Dupli(args_size, args.ptr));
    body.push(Cmd::Call(args_size));
    Ok(Value {
        ptr: ret_ptr,
        ty: ret,
    })
}
