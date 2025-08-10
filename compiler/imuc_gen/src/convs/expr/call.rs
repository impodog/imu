// use super::ExprSolver;
use crate::prelude::*;
use ast::expr::{BinExpr, Expr};
use imuc_ir::sym::ty::{Field, TyItem};
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

    // The initial align used later to adjust the remaining fields
    let init_align =
        config::MEMORY_LAYOUT.update_align_by(config::MEMORY_LAYOUT.init_align(), self_value_size);
    let init_pad = config::memory::align_ptr_to(self_value_size, init_align);
    // The extra space needed for aligning tuples
    let extra_space = init_pad - self_value_size;

    // We do manual tuple alignment here
    let ptr = body.push_stack(init_pad + args_size);
    body.push_void(Cmd::Dupli(self_value.ptr, self_value_size));
    body.push_void(Cmd::Skip(extra_space));
    body.push_void(Cmd::Dupli(args.ptr, args_size));

    use std::iter::once;
    let ty = match &args.ty.kind {
        TyKind::Tuple(args_tuple) => {
            let name = once('(')
                .chain(self_value.ty.name.chars())
                .chain(once(','))
                // NOTE: Here the first paren is skipped, but the last rparen is preserved
                .chain(args.ty.name.chars().skip(1))
                .collect::<String>();
            let name = StrRef::from(name);
            // FIXME: Have to pre-store all the field sizes because the colsure cannot return
            // Result, fix?
            let mut field_sizes = Vec::new();
            for field in args_tuple.0.iter() {
                let field_size = ctx.ty.resolve_or(&field.item, span)?.size_or(span)?;
                field_sizes.push(field_size);
            }
            ctx.ty
                .or_insert_with(name.clone(), || {
                    let mut align = init_align;
                    let mut pad = init_pad;

                    let mut tuple = vec![Field {
                        pad: Ptr::start(),
                        item: TyItem::Solid(self_value.ty.clone()),
                    }];
                    for (field, size) in args_tuple.0.iter().zip(field_sizes.into_iter()) {
                        align = config::MEMORY_LAYOUT.update_align_by(align, size);
                        pad = config::memory::align_ptr_to(pad, align);
                        tuple.push(Field {
                            pad,
                            item: field.item.clone(),
                        });
                        pad += size;
                    }

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
            let args_size = args.ty.size_or(span)?;
            ctx.ty
                .or_insert_with(name.clone(), || {
                    let align = config::MEMORY_LAYOUT.update_align_by(init_align, args_size);
                    let pad = config::memory::align_ptr_to(init_pad, align);
                    let tuple = vec![
                        ir::sym::ty::Field {
                            pad: Ptr::start(),
                            item: self_value.ty.into(),
                        },
                        ir::sym::ty::Field {
                            pad,
                            item: args.ty.into(),
                        },
                    ];
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
    let args_size = args.ty.size_or(span)?;

    // Function pointer
    body.push_cmd(Bytes::ptr(), Cmd::Dupli(Bytes::ptr(), func.ptr));
    // Arguments
    body.push_cmd(args_size, Cmd::Dupli(args_size, args.ptr));
    // Return value
    let ret_ptr = body.push_cmd(ret.size_or(span)?, Cmd::Call(args_size));
    Ok(Value {
        ptr: ret_ptr,
        ty: ret,
    })
}
