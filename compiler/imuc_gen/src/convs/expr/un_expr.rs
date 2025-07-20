use crate::prelude::*;
use ast::expr::UnExpr;

use imuc_ir::sym::ty::TyKind;
use imuc_lexer::token::{ResTy, UnOp};

// TODO: Add hint field
pub struct UnExprConv;

impl Converter for UnExprConv {
    type Input = UnExpr;
}

impl Convert<Value> for UnExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let value: Value = convs::ExprConv::default()
            .convert(ctx, input.val.as_ref())?
            .ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.span())
                        .with_head("UnExpr requires an value"),
                );
                SendError::default()
            })?;
        match input.op {
            UnOp::Ref => {
                let size = value.ty.size().ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(Severity::Error, input.span())
                            .with_head("Taking reference requires a defined and sized type"),
                    );
                    SendError::default()
                })?;
                let body = ctx.body_mut();
                let ptr = body.push_stack(Bytes::ptr());
                body.push(Cmd::Alloc(size));
                body.push(Cmd::WriteHeap(size, Bytes::start(), value.ptr, ptr));

                let name: StrRef = ctx::mangle::mangle_ref(value.ty.name.as_str()).into();
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ref(ir::sym::ty::TyItem::Solid(
                                value.ty.clone(),
                            )),
                            external: true,
                        })
                    })
                    .clone();
                Ok(Value { ptr, ty })
            }
            UnOp::Ptr => {
                let body = ctx.body_mut();
                let ptr = body.push_stack(Bytes::ptr());

                body.push(Cmd::StorePtrAsGlobal(value.ptr));

                let name: StrRef = ctx::mangle::mangle_ptr(value.ty.name.as_str()).into();
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ptr(ir::sym::ty::TyItem::Solid(
                                value.ty.clone(),
                            )),
                            external: true,
                        })
                    })
                    .clone();
                Ok(Value { ptr, ty })
            }
            UnOp::Not => {
                let push_error_fn = ctx.push_error_fn();
                let bytes = value
                    .ty
                    .to_res_ty()
                    .ok_or_else(|| {
                        push_error_fn(ConvError::new(Severity::Error, input.span()).with_text(
                            "Not requires a primitive",
                            format!("Operand type is {}", value.ty.name),
                        ));
                        SendError::default()
                    })?
                    .try_into()?;
                let body = ctx.body_mut();
                let ptr = body.push_stack(value.ty.size_or(input.span()).map_err(push_error_fn)?);
                body.push(Cmd::Not(bytes, value.ptr));
                Ok(Value {
                    ty: value.ty.clone(),
                    ptr,
                })
            }
            UnOp::Neg => {
                let push_error_fn = ctx.push_error_fn();
                let bytes = value
                    .ty
                    .to_res_ty()
                    .filter(|res_ty| {
                        matches!(
                            res_ty,
                            ResTy::I8
                                | ResTy::I16
                                | ResTy::I32
                                | ResTy::I64
                                | ResTy::F32
                                | ResTy::F64
                        )
                    })
                    .ok_or_else(|| {
                        push_error_fn(ConvError::new(Severity::Error, input.span()).with_text(
                            "Neg requires a numeric primitive",
                            format!("Operand type is {}", value.ty.name),
                        ));
                        SendError::default()
                    })?
                    .try_into()?;
                let body = ctx.body_mut();
                let ptr = body.push_stack(value.ty.size_or(input.span()).map_err(push_error_fn)?);
                body.push(Cmd::Neg(bytes, value.ptr));
                Ok(Value {
                    ty: value.ty.clone(),
                    ptr,
                })
            }
            UnOp::Deref => {
                let push_error_fn = ctx.push_error_fn();
                match &value.ty.kind {
                    TyKind::Ptr(ty_item) => {
                        let ty = ctx
                            .ty
                            .resolve_or(ty_item, input.span())
                            .map_err(&push_error_fn)?
                            .clone();
                        let size = ty.size_or(input.span()).map_err(&push_error_fn)?;

                        let ptr = ctx.body_mut().push_stack(size);
                        ctx.body_mut()
                            .push(Cmd::ReadStack(size, Bytes::start(), value.ptr));
                        Ok(Value { ty, ptr })
                    }
                    TyKind::Ref(ty_item) => {
                        let ty = ctx
                            .ty
                            .resolve_or(ty_item, input.span())
                            .map_err(&push_error_fn)?
                            .clone();
                        let size = ty.size_or(input.span()).map_err(&push_error_fn)?;

                        let ptr = ctx.body_mut().push_stack(size);
                        ctx.body_mut()
                            .push(Cmd::ReadHeap(size, Bytes::start(), value.ptr));
                        Ok(Value { ty, ptr })
                    }
                    _ => {
                        push_error_fn(ConvError::new(Severity::Error, input.span()).with_text(
                            "Only Ptr or Ref can be derefed",
                            format!("Input type is {}", value.ty.name),
                        ));
                        Err(SendError::new_error())
                    }
                }
            }
        }
    }
}
