use crate::prelude::*;
use ast::expr::UnExpr;

use imuc_lexer::token::UnOp;

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
                body.push(Cmd::Wrap(size, value.ptr));

                let name: StrRef = format!("@{}", value.ty.name).into();
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
                let global_ptr = ctx.get_global_ptr(value.ptr);

                let body = ctx.body_mut();
                let ptr = body.push_stack(Bytes::global_ptr());

                body.push(Cmd::StoreGlobalPtr(global_ptr));

                let name: StrRef = format!("${}", value.ty.name).into();
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
        }
    }
}
