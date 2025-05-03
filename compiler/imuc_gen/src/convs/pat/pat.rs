use crate::prelude::*;
use ast::pat::*;
use imuc_ir::sym::ty::Cus;
use imuc_lexer::Span;
use ir::sym::ty::{Tuple, TyInner, TyItem, TyKind};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PatConv {
    /// If set to `true`, a missing wildcard type is regarded as an error
    pub requires_ty: bool,
    pub discard_name_warn: bool,
    pub name: Option<StrRef>,
}

impl Converter for PatConv {
    type Input = Pat;
}

impl Convert<Option<Ty>> for PatConv {
    /// Extracts the type info from the pat; [`None`] is only returned if the type is wildcard *and*
    /// [`Self::requires_ty`] is set to `false`
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Ty>> {
        let Self {
            requires_ty,
            discard_name_warn,
            name: given_name,
        } = self;
        let push_error_fn = ctx.push_error_fn();
        let wildcard = |span: &Span| -> Result<Option<Ty>> {
            if requires_ty {
                push_error_fn(
                    ConvError::new(Severity::Error, *span)
                        .with_head("Type wildcard is not allowed here"),
                );
                Err(SendError::new_error())
            } else {
                Ok(None)
            }
        };
        match &**input {
            PatInner::Ident(ident) => {
                if discard_name_warn {
                    ctx.push_error(ConvError::new(Severity::Warn, input.span()).with_head(
                        "Identifier name will be discarded since only type signature is useful",
                    ));
                }
                match ident.ty {
                    Some(ref type_pat) => {
                        convs::TypeConv
                            .convert(ctx, type_pat)
                            .and_then(|option_ty| {
                                if option_ty.is_none() {
                                    wildcard(&input.span)
                                } else {
                                    Ok(option_ty)
                                }
                            })
                    }
                    None => wildcard(&input.span),
                }
            }
            PatInner::Tuple(tuple) => {
                let mut tuple_ty = Vec::new();
                let mut name = String::new();
                name.push('(');
                for sub_pat in tuple.0.iter() {
                    let sub_ty = PatConv {
                        requires_ty,
                        discard_name_warn,
                        name: None,
                    }
                    .convert(ctx, sub_pat)?;
                    if let Some(sub_ty) = sub_ty {
                        name.push_str(sub_ty.name.as_str());
                        name.push(',');
                        tuple_ty.push(TyItem::from(sub_ty));
                    } else {
                        return wildcard(&sub_pat.span);
                    }
                }
                name.pop();
                name.push(')');
                let name = if let Some(given_name) = given_name {
                    given_name
                } else {
                    StrRef::from(name)
                };
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), move || {
                        Ty::new(TyInner::new(name, TyKind::Tuple(Tuple(tuple_ty))))
                    })
                    .clone();
                Ok(Some(ty))
            }
            PatInner::Cus(cus) => {
                let mut cus_ty = BTreeMap::new();
                let name =
                    if let Some(given_name) = given_name {
                        given_name
                    } else {
                        ctx.push_error(ConvError::new(Severity::Error, input.span()).with_head(
                            "Cus type must be given a name and cannot be invoked directly",
                        ));
                        return Err(SendError::new_error());
                    };
                for (sub_name, sub_type) in cus.0.iter() {
                    if let Some(sub_type) = sub_type {
                        let sub_ty = convs::TypeConv.convert(ctx, sub_type)?;
                        if let Some(sub_ty) = sub_ty {
                            cus_ty.insert(sub_name.clone(), TyItem::from(sub_ty));
                        }
                    } else {
                        return wildcard(&input.span);
                    }
                }
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), move || {
                        Ty::new(TyInner::new(name, TyKind::Cus(Cus(cus_ty))))
                    })
                    .clone();
                Ok(Some(ty))
            }
            PatInner::Any(_any) => {
                push_error_fn(
                    ConvError::new(Severity::Error, input.span)
                        .with_head("Any pattern not allowed here"),
                );
                Err(SendError::new_error())
            }
        }
    }
}
