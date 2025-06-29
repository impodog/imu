use crate::prelude::*;
use ast::pat::{PatFlags, Type, TypeKind};
use imuc_lexer::token::ResTy;

pub struct TypeConv;

impl Converter for TypeConv {
    type Input = Type;
}

fn resolve_type_name(ctx: &mut Ctx, name: &str, span: imuc_lexer::Span) -> Result<Ty> {
    // To make name a immutable reference that can be altered
    let mut name = name;
    // extract alias
    let original = ctx.body().get_import(name);
    if let Some(ref original) = original {
        name = original;
    }
    let ty = ctx.get_type(name).ok_or_else(|| {
        ctx.push_error(
            ConvError::new(Severity::Error, span)
                .with_text("Undefined type", format!("Undefined type: {name}")),
        );
        SendError::default()
    })?;
    Ok(ty.to_owned())
}

impl Convert<Option<Ty>> for TypeConv {
    /// Converts an AST type to an actual type; [`None`] is only returned if the type is wildcard
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Ty>> {
        let ty = match &input.kind {
            TypeKind::Wildcard => return Ok(None),
            TypeKind::Single(ref name) => resolve_type_name(ctx, name, input.span())?,
            TypeKind::Prefixed(ast::name::PrefixedName { prefix, name }) => {
                let name = convs::PrefixConv { prefix }.convert(ctx, name)?;
                resolve_type_name(ctx, name.as_str(), input.span())?
            }
            TypeKind::Res(res) => match res {
                ResTy::SelfType => {
                    if let Some(ty) = ctx.body_mut().self_ty() {
                        ty.to_owned()
                    } else {
                        ctx.push_error(
                            ConvError::new(Severity::Error, input.span)
                                .with_head("Self type is required"),
                        );
                        return Err(SendError::default().into());
                    }
                }
                ResTy::Unit => Ty::unit(),
                ResTy::I8 => Ty::i8(),
                ResTy::I16 => Ty::i16(),
                ResTy::I32 => Ty::i32(),
                ResTy::I64 => Ty::i64(),
                ResTy::F32 => Ty::f32(),
                ResTy::F64 => Ty::f64(),
                ResTy::Bool => Ty::bool(),
                ResTy::Str => Ty::str(),
                ResTy::Ptr => Ty::ptr(),
            },
            TypeKind::Tuple(tuple) => {
                let mut list = Vec::new();
                for pat in tuple.iter() {
                    list.push(TypeConv.convert(ctx, pat)?.ok_or_else(|| {
                        ctx.push_error(
                            ConvError::new(Severity::Error, pat.span())
                                .with_head("Wildcard types not allowed in solid type"),
                        );
                        SendError::new_error()
                    })?);
                }
                let name: StrRef =
                    ctx::mangle::mangle_tuple_name(list.iter().map(|ty| ty.name.as_str())).into();
                ctx.ty
                    .or_insert_with(name.clone(), move || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Tuple(ir::sym::ty::Tuple(
                                list.into_iter().map(ir::sym::ty::TyItem::Solid).collect(),
                            )),
                            external: false,
                        })
                    })
                    .clone()
            }
        };
        let ty = match input.flags {
            PatFlags::Unique => ty,
            PatFlags::Shared => {
                let name: StrRef = ctx::mangle::mangle_ref(ty.name.as_str()).into();
                ctx.ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ref(ir::sym::ty::TyItem::Solid(ty)),
                            external: false,
                        })
                    })
                    .clone()
            }
            PatFlags::Stack => {
                let name: StrRef = ctx::mangle::mangle_ptr(ty.name.as_str()).into();
                ctx.ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ptr(ir::sym::ty::TyItem::Solid(ty)),
                            external: false,
                        })
                    })
                    .clone()
            }
        };
        Ok(Some(ty))
    }
}
