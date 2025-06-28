use crate::prelude::*;
use ast::expr::Cus;
use ir::sym::ty::TyItem;
use std::collections::BTreeMap;

pub struct CusExprConv;

impl Converter for CusExprConv {
    type Input = Cus;
}

impl Convert<Value> for CusExprConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let ty = convs::TypeConv.convert(ctx, &input.ty)?.ok_or_else(|| {
            ctx.push_error(
                ConvError::new(Severity::Error, input.span)
                    .with_head("A solid type is required for Cus instantiation"),
            );
            SendError::default()
        })?;
        if let ir::sym::ty::TyKind::Cus(cus) = &ty.kind {
            let mut map = BTreeMap::new();
            for (name, expr) in input.elem.iter() {
                // TODO: Hint the type with cus item
                let value = convs::ExprConv::default()
                    .convert(ctx, expr)?
                    .ok_or_else(|| {
                        ctx.push_error(
                            ConvError::new(Severity::Error, input.span)
                                .with_head("A value is required for Cus field instantiation"),
                        );
                        SendError::default()
                    })?;
                map.insert(name.to_owned(), value);
            }
            let mut size = Bytes::default();
            for (key, value) in map.iter() {
                if let Some(ty) = cus.0.get(key) {
                    match ty {
                        TyItem::Solid(ty) => {
                            if !ty.test_eq(&value.ty) {
                                ctx.push_error(
                                    ConvError::new(Severity::Error, input.span).with_text(
                                        "Types mismatch in Cus field instantiation",
                                        format!("required {}, found {}", ty.name, value.ty.name),
                                    ),
                                );
                                return Err(SendError::default().into());
                            }
                        }
                        TyItem::Pending(name) => {
                            ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                                "Type is undefined in Cus field instantiation (FIXME)",
                                format!("Type {name} is undefined"),
                            ));
                            return Err(SendError::default().into());
                        }
                    }
                } else {
                    ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                        "Unknown field in Cus field instantiation",
                        format!("Unknown field: {key}"),
                    ));
                    return Err(SendError::default().into());
                }
                size += value.ty.size_or(input.span).map_err(ctx.push_error_fn())?;
            }
            for key in cus.0.keys() {
                if !map.contains_key(key) {
                    ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                        "Missing field int Cus field instantiation",
                        format!("Missing field: {key}"),
                    ));
                    return Err(SendError::default().into());
                }
            }

            let push_error_fn = ctx.push_error_fn();
            let body = ctx.body_mut();
            for (_name, value) in map.into_iter() {
                body.push(Cmd::Dupli(
                    value.ty.size_or(input.span).map_err(&push_error_fn)?,
                    value.ptr,
                ));
            }

            let ptr = body.push_stack(size);
            Ok(Value { ty, ptr })
        } else {
            ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                "Cus type required in Cus instantiation",
                format!("Found {}", ty.name),
            ));
            Err(SendError::default().into())
        }
    }
}
