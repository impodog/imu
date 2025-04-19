use crate::prelude::*;
use ast::{expr::Body, item::Fun, pat::Pat};
use ir::sym::ty::{TyInner, TyKind};

pub struct FunConv {
    pub name: StrRef,
    pub self_ty: Option<Ty>,
}

impl Converter for FunConv {
    type Input = Fun;
}

impl Convert<Value> for FunConv {
    /// Converts the function
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let FunConv { name, self_ty } = self;
        ctx.push_body(name.as_str(), self_ty);
        let value = convs::BodyConv.convert(ctx, &input.body)?;
        let body = ctx
            .pop_body()
            .expect("a body should exist after pushing")
            .take_cmd();
        let param: Ty = todo!("Parse parameters!");
        let ret = if let Some(ref ret) = input.ret {
            convs::TypeConv.convert(ctx, ret)?
        } else {
            None
        }
        .unwrap_or_else(Ty::unit);
        if !ret.test_eq(&value.ty) {
            // TODO: Should this be warn?
            ctx.push_error(ConvError::new(Severity::Warn, input.span()).with_text(
                "Fun return types mismatch",
                format!("Expected {}, but returned {}", ret.name, value.ty.name),
            ));
            return Err(SendError::default().into());
        }
        let ty = Ty::new(TyInner {
            name: ctx::mangle::mangle_fun_ty(name.as_str()).into(),
            kind: TyKind::Fun {
                param: param.into(),
                ret: ret.into(),
            },
            external: false,
        });
        let ptr_ty = Ty::new(TyInner {
            name: ctx::mangle::mangle_ptr(ty.name.as_str()).into(),
            kind: TyKind::Ptr(ty.into()),
            external: false,
        });
        ctx.globs
            .write()
            .unwrap()
            .load_fun(ctx.bottom_mut(), name, ptr_ty);
    }
}
