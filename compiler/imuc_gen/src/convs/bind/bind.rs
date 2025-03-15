use crate::prelude::*;
use ast::bind::{Bind, Let};
use ast::expr::Expr;
use ast::pat::{IdentKind, IdentPat, Pat, PatInner};
use ir::sym::ty::TyKind;

pub struct BindConv;

impl Converter for BindConv {
    type Input = Bind;
}

enum Conversion<'a> {
    Value(Option<Value>),
    Expr(&'a Expr, Option<Ty>),
}

impl Conversion<'_> {
    fn with_hint(self, hint: Option<Ty>) -> Self {
        match self {
            Self::Expr(expr, _) => Self::Expr(expr, hint),
            _ => self,
        }
    }

    fn into_value(self, ctx: &mut Ctx) -> Result<Option<Value>> {
        match self {
            Self::Value(value) => Ok(value),
            Self::Expr(expr, hint) => convs::ExprConv::default()
                .with_hint(hint)
                .convert(ctx, expr),
        }
    }
}

fn convert_let(ctx: &mut Ctx, pat: &Pat, val: Conversion) -> Result<()> {
    match &**pat {
        PatInner::Ident(ident) => {
            let ty = if let Some(ref ty) = ident.ty {
                convs::TypeConv.convert(ctx, ty)?
            } else {
                None
            };
            let value = val.with_hint(ty).into_value(ctx)?;
            let body = ctx.body_mut();
            match &ident.ident {
                IdentKind::Unused => Ok(()),
                IdentKind::Value(name) => {
                    if let Some(value) = value {
                        if let Some(drop_value) =
                            body.locals_mut().value.insert(name.clone(), value)
                        {
                            body.drop_value(&drop_value)?;
                        }
                        Ok(())
                    } else {
                        Err(errors::ConvError::ValueRequired("let binding".to_owned()).into())
                    }
                }
            }
        }
        PatInner::Tuple(tuple) => {
            let value = val
                .into_value(ctx)?
                .ok_or_else(|| errors::ConvError::ValueRequired("let binding".to_owned()))?;
            match &value.ty.kind {
                TyKind::Tuple(tuple_ty) => {
                    if tuple.0.len() != tuple_ty.0.len() {
                        Err(errors::ConvError::TypesMismatch(format!(
                            "{} with a pattern of {}",
                            value.ty.name,
                            tuple.0.len()
                        ))
                        .into())
                    } else {
                        let mut ptr = value.ptr;
                        for (pat, ty) in tuple.0.iter().zip(tuple_ty.0.iter()) {
                            let ty = ctx.ty.resolve_or(ty)?.clone();
                            convert_let(
                                ctx,
                                pat,
                                Conversion::Value(Some(Value::new(ty.clone(), ptr))),
                            )?;
                            ptr += ty.size_or()?;
                        }
                        Ok(())
                    }
                }
                _ => Err(errors::ConvError::TypesMismatch(format!(
                    "{} with a tuple binding",
                    value.ty.name
                ))
                .into()),
            }
        }
        PatInner::Named(named) => {
            let value = val
                .into_value(ctx)?
                .ok_or_else(|| errors::ConvError::ValueRequired("let binding".to_owned()))?;
            match &value.ty.kind {
                TyKind::Cus(cus) => {
                    let mut left = cus.0.iter();
                    let mut ptr = value.ptr;
                    for (name, item) in named.0.iter() {
                        let (current_ptr, ty) = loop {
                            if let Some((origin_name, origin_item)) = left.next() {
                                let origin_ty = ctx.ty.resolve_or(origin_item)?.clone();
                                let prev = ptr;
                                ptr += origin_ty.size_or()?;
                                if name == origin_name {
                                    break (prev, origin_ty);
                                }
                            } else {
                                return Err(
                                    errors::ConvError::UnexpectedField(name.to_string()).into()
                                );
                            }
                        };
                        if let Some(named_ty) = item {
                            if let Some(named_ty) = convs::TypeConv.convert(ctx, named_ty)? {
                                if !named_ty.test_eq(&ty) {
                                    return Err(errors::ConvError::TypesMismatch(format!(
                                        "{} and {}",
                                        named_ty.name, ty.name
                                    ))
                                    .into());
                                }
                            }
                        }
                        let pat = Pat::new(PatInner::Ident(IdentPat {
                            ident: IdentKind::Value(name.clone()),
                            ty: None,
                        }));
                        convert_let(
                            ctx,
                            &pat,
                            Conversion::Value(Some(Value {
                                ptr: current_ptr,
                                ty,
                            })),
                        )?;
                    }
                    Ok(())
                }
                _ => Err(errors::ConvError::TypesMismatch(format!(
                    "{} with a named binding",
                    value.ty.name
                ))
                .into()),
            }
        }
        PatInner::Any(_any) => {
            Err(errors::ConvError::SyntaxMaybeImplement("any pat in let binding".to_owned()).into())
        }
    }
}

impl Convert<()> for BindConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        match input {
            Bind::Let(bind) => {
                let Let { pat, val } = bind;
                convert_let(ctx, pat, Conversion::Expr(val, None))
            }
            Bind::Item(item) => {
                todo!()
            }
        }
    }
}
