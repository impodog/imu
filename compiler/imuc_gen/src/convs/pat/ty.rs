use crate::prelude::*;
use ast::pat::{PatFlags, Type, TypeKind};
use imuc_lexer::token::ResTy;

pub struct TypeConv;

impl Converter for TypeConv {
    type Input = Type;
}

impl Convert<Option<Ty>> for TypeConv {
    /// Converts an AST type to an actual type; [`None`] is only returned if the type is wildcard
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Ty>> {
        let ty = match &input.kind {
            TypeKind::Wildcard => return Ok(None),
            TypeKind::Single(name) => {
                let ty = ctx.get_type(name).ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(Severity::Error, input.span)
                            .with_text("Undefined type", format!("Undefined type: {}", name)),
                    );
                    SendError::default()
                })?;
                ty.to_owned()
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
                ResTy::Drop => Ty::drop(),
            },
        };
        let ty = match input.flags {
            PatFlags::Unique => ty,
            PatFlags::Shared => {
                let name: StrRef = format!("@{}", ty.name).into();
                ctx.ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ref(ir::sym::ty::TyItem::Solid(ty)),
                            external: true,
                        })
                    })
                    .clone()
            }
            PatFlags::Stack => {
                let name: StrRef = format!("${}", ty.name).into();
                ctx.ty
                    .or_insert_with(name.clone(), || {
                        Ty::new(ir::sym::ty::TyInner {
                            name,
                            kind: ir::sym::ty::TyKind::Ptr(ir::sym::ty::TyItem::Solid(ty)),
                            external: true,
                        })
                    })
                    .clone()
            }
        };
        Ok(Some(ty))
    }
}
