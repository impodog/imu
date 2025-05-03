use super::ExprSolver;
use crate::prelude::*;
use ast::expr::Call;
use ir::sym::ty::TyKind;

#[derive(Default)]
pub struct CallConv {
    pub solver: ExprSolver,
}

impl Converter for CallConv {
    type Input = Call;
}

impl Convert<Value> for CallConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let func = convs::ExprConv::default()
            .convert(ctx, &input.func)?
            .ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.func.unwrap_span())
                        .with_head("Required fun pointer for function call"),
                );
                SendError::new_error()
            })?;
        let args = convs::ExprConv::default()
            .convert(ctx, &input.args)?
            .ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.func.unwrap_span())
                        .with_head("Required arguments for function call"),
                );
                SendError::new_error()
            })?;
        let ret = match &func.ty.kind {
            TyKind::Ptr(item) => {
                let ty = ctx.ty.resolve_or(item, input.span())?.clone();
                match &ty.kind {
                    TyKind::Fun { param, ret } => {
                        let param = ctx.ty.resolve_or(param, input.span())?.clone();
                        let ret = ctx.ty.resolve_or(ret, input.span())?.clone();
                        if !args.ty.test_eq(&param) {
                            ctx.push_error(
                                ConvError::new(Severity::Error, input.span()).with_text(
                                    "Function parameter and argument types mismatch",
                                    format!("Expected {}, given {}", param.name, args.ty.name),
                                ),
                            );
                            return Err(SendError::new_error());
                        }
                        ret
                    }
                    _ => {
                        ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                            "Expected function to be called",
                            format!("Given value type is {}", func.ty.name),
                        ));
                        return Err(SendError::new_error());
                    }
                }
            }
            _ => {
                ctx.push_error(ConvError::new(Severity::Error, input.span).with_text(
                    "Expected function pointer",
                    format!("Given value type is {}", func.ty.name),
                ));
                return Err(SendError::new_error());
            }
        };
        // NOTE: This data copy is required regardless of whether the copy is needed, to make sure the
        // data stays in place of a function call, and will be optimized later
        let body = ctx.body_mut();
        // For function pointer
        body.push_stack(Bytes::ptr());
        // For arguments
        let args_size = args.ty.size_or(input.span())?;
        body.push_stack(args_size);
        // For return value
        let ret_ptr = body.push_stack(ret.size_or(input.span())?);
        body.push(Cmd::Dupli(Bytes::ptr(), func.ptr));
        body.push(Cmd::Dupli(args_size, args.ptr));
        body.push(Cmd::Call(args_size));
        Ok(Value {
            ptr: ret_ptr,
            ty: ret,
        })
    }
}
