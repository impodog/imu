use crate::prelude::*;
use ast::{item::Fun, module::Public};
use convs::bind::binds::*;
use ir::{
    cmd::CmdBody,
    sym::{
        ty::{TyInner, TyKind},
        Fun as IrFun, FunSig,
    },
};

pub struct FunConv {
    pub name: StrRef,
    pub public: Public,
    pub self_ty: Option<Ty>,
}

impl Converter for FunConv {
    type Input = Fun;
}

impl Convert<()> for FunConv {
    /// Converts the function definition
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let FunConv {
            name,
            // TODO: Add publicity handling
            public,
            self_ty,
        } = self;

        // Parse parameter and return types signature
        let ret = if let Some(ref ret) = input.ret {
            convs::TypeConv.convert(ctx, ret)?
        } else {
            None
        }
        .unwrap_or_else(Ty::unit);
        let param = convs::PatConv {
            requires_ty: true,
            discard_name_warn: false,
            name: Some(ctx::mangle::mangle_fun_sig(name.as_str()).into()),
        }
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

        // Work to compile body expression
        ctx.push_body(name.as_str(), self_ty, false);
        // Assign arguments to current namespace
        convert_let(
            ctx,
            &input.param,
            Conversion::Value(Some(Value::new(param.clone(), Bytes::start()))),
        )?;
        let value = convs::BodyConv.convert(ctx, &input.body)?;
        let body = ctx
            .pop_body()
            .expect("a body should exist after pushing")
            .take_cmd();

        if !ret.test_eq(&value.ty) {
            ctx.push_error(ConvError::new(Severity::Warn, input.span()).with_text(
                "Fun return types mismatch",
                format!("Expected {}, but returned {}", ret.name, value.ty.name),
            ));
            return Err(SendError::default().into());
        }

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
