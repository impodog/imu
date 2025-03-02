use crate::prelude::*;
use ast::expr::Cus;
use ir::sym::ty::TyItem;
use std::collections::BTreeMap;

pub struct CusConv;

impl Converter for CusConv {
    type Input = Cus;
}

impl Convert<Value> for CusConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let ty = convs::TypeConv
            .convert(ctx, &input.ty)?
            .ok_or_else(|| errors::ConvError::TypeRequired("Cus".to_string()))?;
        if let ir::sym::ty::TyKind::Cus(cus) = &ty.kind {
            let mut map = BTreeMap::new();
            for (name, expr) in input.elem.iter() {
                // TODO: Hint the type with cus item
                let value = convs::ExprConv::default()
                    .convert(ctx, expr)?
                    .ok_or_else(|| errors::ConvError::ValueRequired("Cus field".to_owned()))?;
                map.insert(name.to_owned(), value);
            }
            let mut size = Bytes::default();
            for (key, value) in map.iter() {
                if let Some(ty) = cus.0.get(key) {
                    match ty {
                        TyItem::Solid(ty) => {
                            if !ty.test_eq(&value.ty) {
                                return Err(errors::ConvError::TypesMismatch(format!(
                                    "required {}, found {}",
                                    ty.name, value.ty.name
                                ))
                                .into());
                            }
                        }
                        TyItem::Pending(name) => {
                            return Err(errors::ConvError::UndefinedType(name.to_string()).into())
                        }
                    }
                } else {
                    return Err(errors::ConvError::UnknownField(key.to_string()).into());
                }
                size += value.ty.size_or()?;
            }
            for key in cus.0.keys() {
                if !map.contains_key(key) {
                    return Err(errors::ConvError::MissingField(key.to_string()).into());
                }
            }

            let body = ctx.body_mut();
            for (_name, value) in map.into_iter() {
                body.push(Cmd::Dupli(value.ty.size_or()?, value.ptr));
            }

            let ptr = body.push_stack(size);
            Ok(Value { ty, ptr })
        } else {
            Err(errors::ConvError::CusRequired("Cus".to_string()).into())
        }
    }
}
