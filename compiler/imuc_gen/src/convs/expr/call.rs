use super::ExprSolver;
use crate::prelude::*;
use ast::expr::{Call, Expr};
use ir::sym::ty::{Ty, TyKind, TyInner}

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
        match &func.ty.kind {
            TyKind::Ptr(item) => {
                match item {
                    // TODO: Work function call
                }
            }

        }
    }
}
