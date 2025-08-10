use super::ExprSolver;
use crate::prelude::*;
use ast::expr::Req;
use imuc_ir::sym::ty::TyKind;

pub struct ReqConv {
    pub solver: ExprSolver,
}

impl Converter for ReqConv {
    type Input = Req;
}

impl Convert<Value> for ReqConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let Self { solver } = self;
        let (hint, param, ret) = match solver.hint {
            Some(hint) => {
                match hint.kind {
                    TyKind::Ptr(ref ty_item) => {
                        let item = ctx.ty.resolve_or(ty_item, input.span()).map_err(|err| {
                            ctx.push_error(err);
                            SendError::new_error()
                        })?;
                        match item.kind {
                            TyKind::Fun { ref param, ref ret } => {
                                let push_error_fn = ctx.push_error_fn();
                                let param = ctx
                                    .ty
                                    .resolve_or(param, input.span())
                                    .map_err(&push_error_fn)?
                                    .size_or(input.span())
                                    .map_err(&push_error_fn)?;
                                let ret = ctx
                                    .ty
                                    .resolve_or(ret, input.span())
                                    .map_err(&push_error_fn)?
                                    .size_or(input.span())
                                    .map_err(&push_error_fn)?;
                                (hint, param, ret)
                            }
                            _ => {
                                ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                                    "Expected requirement hint type to be pointer to a function",
                                    format!("Found {}", hint.name),
                                ));
                                return Err(SendError::new_error());
                            }
                        }
                    }
                    _ => {
                        ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                            "Expected requirement hint type to be pointer",
                            format!("Found {}", hint.name),
                        ));
                        return Err(SendError::new_error());
                    }
                }
            }
            None => {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.span())
                        .with_head("No hint type given to this requirement function"),
                );
                return Err(SendError::new_error());
            }
        };

        // Add to globals and prevent multiple additions, also mangles the name to prevent naming
        // conflicts and users can not use it without "req" keyword wrap
        // NOTE: Runtimes must also use this naming mangling convention
        let name = StrRef::from(ctx::mangle::mangle_req(input.name.as_str()));

        let globs = ctx.globs.clone();
        let mut globs_lock = globs.write().unwrap();
        // Prevent repeated addition
        let glob = if let Some(glob) = globs_lock.get(&name) {
            glob
        } else {
            globs_lock.load_fun_check(ctx.bottom_mut(), name.clone(), hint.clone(), param, ret);
            globs_lock
                .get(&name)
                .expect("Should contain the global after inserting")
        };

        let ptr = ctx
            .body_mut()
            .push_cmd(Bytes::ptr(), Cmd::StorePtr(glob.ptr()));
        Ok(Value { ptr, ty: hint })
    }
}
