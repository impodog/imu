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
    /// If set to `true`, a warning will be thrown out when the type contains identifier patterns
    /// with identifier names (except identifiers in a cus pattern)
    pub discard_name_warn: bool,
    pub name: Option<StrRef>,
    pub public: ast::module::Public,
}

impl Converter for PatConv {
    type Input = Pat;
}

impl Convert<Option<Ty>> for PatConv {
    /// Extracts the type info from the pat; `None` is only returned if the type is wildcard *and*
    /// `Self::requires_ty` is set to `false`
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Ty>> {
        let Self {
            requires_ty,
            discard_name_warn,
            name: given_name,
            public,
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
                if tuple.0.is_empty() {
                    // NOTE: Patterns can be empty, because parser did not handle this edge case
                    return Ok(Some(Ty::unit()));
                }

                let mut tuple_ty = Vec::new();
                let mut name = String::new();
                let mut pad = Bytes::start();
                let mut align = config::MEMORY_LAYOUT.init_align();
                name.push('(');
                for sub_pat in tuple.0.iter() {
                    let sub_ty = PatConv {
                        requires_ty,
                        discard_name_warn,
                        name: None,
                        public,
                    }
                    .convert(ctx, sub_pat)?;
                    if let Some(sub_ty) = sub_ty {
                        name.push_str(sub_ty.name.as_str());
                        name.push(',');

                        let sub_ty_size = sub_ty.size_or(sub_pat.span()).map_err(&push_error_fn)?;
                        align = config::MEMORY_LAYOUT.update_align_by(align, sub_ty_size);
                        pad = config::memory::align_ptr_to(pad, align);
                        tuple_ty.push(ir::sym::ty::Field {
                            pad,
                            item: TyItem::from(sub_ty),
                        });
                        pad += sub_ty_size;
                    } else {
                        return wildcard(&sub_pat.span);
                    }
                }
                // Only pop trailing commas if any
                if name != "(" {
                    name.pop();
                }
                name.push(')');
                let name = if let Some(given_name) = given_name {
                    given_name
                } else {
                    StrRef::from(name)
                };
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), move || {
                        Ty::new(TyInner::new_with_public(
                            public,
                            name,
                            TyKind::Tuple(Tuple(tuple_ty)),
                        ))
                    })
                    .clone();
                Ok(Some(ty))
            }
            PatInner::Cus(cus) => {
                let mut fields = Vec::new();
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
                            let size = sub_ty.size_or(sub_type.span()).map_err(&push_error_fn)?;
                            fields.push((sub_name.clone(), TyItem::from(sub_ty), size));
                        } else {
                            return wildcard(&sub_type.span());
                        }
                    } else {
                        return wildcard(&input.span);
                    }
                }

                // Optimize padding by small fields first
                fields.sort_by_key(|(_, _, size)| size.num());
                // Handle padding
                let mut cus_ty = BTreeMap::new();
                let mut pad = Bytes::start();
                let mut align = config::MEMORY_LAYOUT.init_align();
                for (sub_name, sub_ty, size) in fields.into_iter() {
                    align = config::MEMORY_LAYOUT.update_align_by(align, size);
                    pad = config::memory::align_ptr_to(pad, align);
                    cus_ty.insert(sub_name, ir::sym::ty::Field { pad, item: sub_ty });
                    pad += size;
                }
                let ty = ctx
                    .ty
                    .or_insert_with(name.clone(), move || {
                        Ty::new(TyInner::new_with_public(
                            public,
                            name,
                            TyKind::Cus(Cus(cus_ty)),
                        ))
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
