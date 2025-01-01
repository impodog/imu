use crate::prelude::*;
use ast::expr::UnExpr;
use ctx::Value;
use imuc_lexer::token::UnOp;

pub struct UnExprConv;

impl Converter for UnExprConv {
    type Input = UnExpr;
}

impl Convert<Option<Value>> for UnExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        let value: Value = convs::ExprConv
            .convert(ctx, input.val.as_ref())?
            .ok_or_else(|| errors::ConvError::ValueRequired("UnExpr".to_owned()))?;
        match input.op {
            UnOp::Ref => {
                let body = ctx.body_mut_or()?;
                let ptr = body.push_stack(Bytes::ptr());
                body.push(Cmd::Wrap(
                    value.ty.size().ok_or_else(|| {
                        errors::ConvError::UndefinedType(value.ty.name.to_string())
                    })?,
                    value.ptr,
                ));

                let name: StrRef = format!("@{}", value.ty.name).into();
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ref(ir::sym::ty::TyItem::Solid(
                                value.ty.clone(),
                            )),
                            external: value.ty.external,
                        })
                    })
                    .clone();
                Ok(Some(Value { ptr, ty }))
            }
            UnOp::Not => {
                let body = ctx.body_mut_or()?;
                let bytes = value
                    .ty
                    .to_res_ty()
                    .ok_or_else(|| errors::ConvError::PrimitiveRequired("Not".to_owned()))?
                    .try_into()?;
                let ptr = body.push_stack(value.ty.size_or()?);
                body.push(Cmd::Not(bytes, value.ptr));
                Ok(Some(Value {
                    ty: value.ty.clone(),
                    ptr,
                }))
            }
        }
    }
}
