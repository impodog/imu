use crate::prelude::*;
use ast::{item::Fun, module::Public};
use imuc_ir::{
    cmd::CmdBody,
    sym::{Fun as IrFun, FunSig},
};
use ir::sym::ty::{TyInner, TyKind};

pub struct FunConv {
    pub name: StrRef,
    pub public: Public,
    pub self_ty: Option<Ty>,
}

impl Converter for FunConv {
    type Input = Fun;
}

impl Convert<()> for FunConv {
    /// Converts the function
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let FunConv {
            name,
            // TODO: Add publicity handling
            public,
            self_ty,
        } = self;
        ctx.push_body(name.as_str(), self_ty);
        let value = convs::BodyConv.convert(ctx, &input.body)?;
        let body = ctx
            .pop_body()
            .expect("a body should exist after pushing")
            .take_cmd();

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
        let param = convs::PatConv { requires_ty: true }
            .convert(ctx, &input.param)?
            .expect("PatConv should not return None when requires_ty is enabled");
        let fun_ty_name = StrRef::from(ctx::mangle::mangle_fun_ty(name.as_str()));
        let fun_ty = ctx
            .ty
            .or_insert_with(fun_ty_name.clone(), || {
                Ty::new(TyInner {
                    name: fun_ty_name,
                    kind: TyKind::Fun {
                        param: param.clone().into(),
                        ret: ret.clone().into(),
                    },
                    external: false,
                })
            })
            .clone();
        let ptr_ty_name = StrRef::from(ctx::mangle::mangle_ptr(fun_ty.name.as_str()));
        let ptr_ty = ctx
            .ty
            .or_insert_with(ptr_ty_name.clone(), move || {
                Ty::new(TyInner {
                    name: ptr_ty_name,
                    kind: TyKind::Ptr(fun_ty.into()),
                    external: false,
                })
            })
            .clone();
        let globs = ctx.globs.clone();
        globs
            .write()
            .unwrap()
            .load_fun(ctx.bottom_mut(), name.clone(), ptr_ty);

        ctx.fun.insert(
            name.clone(),
            IrFun {
                name,
                sig: FunSig { ret, param },
                body: CmdBody::new(body),
            },
        );

        Ok(())
    }
}
