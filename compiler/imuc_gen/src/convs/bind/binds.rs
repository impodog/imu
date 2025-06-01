use crate::prelude::*;
use ast::bind::{Bind, Let};
use ast::expr::Expr;
use ast::pat::{IdentKind, IdentPat, Pat, PatInner};
use ir::sym::ty::TyKind;

pub struct BindConv;

impl Converter for BindConv {
    type Input = Bind;
}

pub enum Conversion<'a> {
    Value(Option<Value>),
    Expr(&'a Expr, Option<Ty>),
}

impl Conversion<'_> {
    pub fn with_hint(self, hint: Option<Ty>) -> Self {
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

pub fn convert_let(ctx: &mut Ctx, pat: &Pat, val: Conversion) -> Result<()> {
    match &**pat {
        PatInner::Ident(ident) => {
            let ty = if let Some(ref ty) = ident.ty {
                convs::TypeConv.convert(ctx, ty)?
            } else {
                None
            };
            let value = val.with_hint(ty).into_value(ctx)?;
            match &ident.ident {
                IdentKind::Unused => Ok(()),
                IdentKind::Value(name) => {
                    if let Some(value) = value {
                        ctx.body_mut()
                            .locals_mut()
                            .value
                            .insert(name.clone(), value);
                        Ok(())
                    } else {
                        ctx.push_error(
                            ConvError::new(Severity::Error, pat.span())
                                .with_head("Let binding to a name requires a value"),
                        );
                        Err(SendError::default().into())
                    }
                }
            }
        }
        PatInner::Tuple(tuple) => {
            let value = val.into_value(ctx)?.ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, pat.span())
                        .with_head("Tuple binding requires a value"),
                );
                SendError::default()
            })?;
            match &value.ty.kind {
                TyKind::Tuple(tuple_ty) => {
                    if tuple.0.len() != tuple_ty.0.len() {
                        ctx.push_error(ConvError::new(Severity::Error, pat.span()).with_text(
                            "Tuple pattern length mismatch",
                            format!(
                                "Lengths are {} of type and {} of value",
                                tuple_ty.0.len(),
                                tuple.0.len()
                            ),
                        ));
                        Err(SendError::default().into())
                    } else {
                        let mut ptr = value.ptr;
                        for (pat, ty) in tuple.0.iter().zip(tuple_ty.0.iter()) {
                            let ty = ctx
                                .ty
                                .resolve_or(ty, pat.span)
                                .map_err(ctx.push_error_fn())?
                                .clone();
                            convert_let(
                                ctx,
                                pat,
                                Conversion::Value(Some(Value::new(ty.clone(), ptr))),
                            )?;
                            ptr += ty.size_or(pat.span()).map_err(ctx.push_error_fn())?;
                        }
                        Ok(())
                    }
                }
                _ => {
                    ctx.push_error(ConvError::new(Severity::Error, pat.span()).with_text(
                        "Tuple binding requires a tuple value",
                        format!("Found {}", value.ty.name),
                    ));
                    Err(SendError::default().into())
                }
            }
        }
        PatInner::Cus(named) => {
            let value = val.into_value(ctx)?.ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, pat.span())
                        .with_head("Cut binding requires a value"),
                );
                SendError::default()
            })?;
            match &value.ty.kind {
                TyKind::Cus(cus) => {
                    let mut left = cus.0.iter();
                    let mut ptr = value.ptr;
                    for (name, item) in named.0.iter() {
                        let (current_ptr, ty) = loop {
                            if let Some((origin_name, origin_item)) = left.next() {
                                let origin_ty = ctx
                                    .ty
                                    .resolve_or(origin_item, pat.span())
                                    .map_err(ctx.push_error_fn())?
                                    .clone();
                                let prev = ptr;
                                ptr +=
                                    origin_ty.size_or(pat.span()).map_err(ctx.push_error_fn())?;
                                if name == origin_name {
                                    break (prev, origin_ty);
                                }
                            } else {
                                ctx.push_error(
                                    ConvError::new(Severity::Error, pat.span()).with_text(
                                        "Unexpected field in Cus binding",
                                        format!("Unexpected field: {}", name),
                                    ),
                                );
                                return Err(SendError::default().into());
                            }
                        };
                        if let Some(cus_ty) = item {
                            if let Some(named_ty) = convs::TypeConv.convert(ctx, cus_ty)? {
                                if !named_ty.test_eq(&ty) {
                                    ctx.push_error(
                                        ConvError::new(Severity::Error, pat.span()).with_text(
                                            "Cus field types mismatch",
                                            format!(
                                                "Field {} requires {}, but {} is given",
                                                name, named_ty.name, ty.name
                                            ),
                                        ),
                                    );
                                    return Err(SendError::default().into());
                                }
                            }
                        }
                        let pat = Pat::new(
                            PatInner::Ident(IdentPat {
                                ident: IdentKind::Value(name.clone()),
                                ty: None,
                            }),
                            Default::default(),
                        );
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
                _ => {
                    ctx.push_error(ConvError::new(Severity::Error, pat.span()).with_text(
                        "Cus binding requires a cus value",
                        format!("Found {}", value.ty.name),
                    ));
                    Err(SendError::default().into())
                }
            }
        }
        PatInner::Any(_any) => {
            todo!("Should we implement any binding?")
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
            Bind::Item(item) => convs::ItemConv.convert(ctx, item),
        }
    }
}
