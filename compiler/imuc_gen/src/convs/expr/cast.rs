use crate::prelude::*;
use ast::expr::Cast;
use imuc_lexer::token::ResTy;

pub struct CastConv;

impl Converter for CastConv {
    type Input = Cast;
}

enum CastDest {
    Integer(NumBytes),
    Float(NumBytes),
}

impl Convert<Value> for CastConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let value = convs::ExprConv::default()
            .convert(ctx, input.expr.as_ref())?
            .ok_or_else(|| {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.expr.unwrap_span())
                        .with_head("Required a value for conversion"),
                );
                SendError::new_error()
            })?;
        let dst = match input.ty {
            ResTy::I8 | ResTy::I16 | ResTy::I32 | ResTy::I64 => CastDest::Integer(
                NumBytes::try_from(input.ty).expect("integer ResTy should be sized"),
            ),
            ResTy::F32 | ResTy::F64 => {
                CastDest::Float(NumBytes::try_from(input.ty).expect("float ResTy should be sized"))
            }
            _ => {
                ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                    "Expected numeric primitive",
                    format!("Found {:?}", input.ty),
                ));
                return Err(SendError::new_error());
            }
        };
        let dst_bytes = Bytes::from(match dst {
            CastDest::Integer(num_bytes) => num_bytes,
            CastDest::Float(num_bytes) => num_bytes,
        });
        let ty = Ty::from_res(input.ty).expect("numeric ResTy should have corresponding Ty");

        let body = ctx.body_mut();
        match value.ty.kind {
            ir::sym::ty::TyKind::Res(res_ty) => {
                if res_ty == input.ty {
                    ctx.push_error(
                        ConvError::new(Severity::Warn, input.span())
                            .with_head("Cast to the same type has no effect"),
                    );
                    return Ok(value);
                }

                match res_ty {
                    ResTy::Bool | ResTy::Ptr | ResTy::I8 | ResTy::I16 | ResTy::I32 | ResTy::I64 => {
                        let src =
                            NumBytes::try_from(res_ty).expect("integer ResTy should be sized");
                        match dst {
                            CastDest::Integer(dst) => {
                                let ptr = body.push_stack(dst_bytes);
                                body.push(Cmd::IToI(src, dst, value.ptr));
                                Ok(Value { ptr, ty })
                            }
                            CastDest::Float(dst) => {
                                let ptr = body.push_stack(dst_bytes);
                                body.push(Cmd::IToF(src, dst, value.ptr));
                                Ok(Value { ptr, ty })
                            }
                        }
                    }
                    ResTy::F32 | ResTy::F64 => {
                        let src = NumBytes::try_from(res_ty).expect("float ResTy should be sized");
                        match dst {
                            CastDest::Integer(dst) => {
                                let ptr = body.push_stack(dst_bytes);
                                body.push(Cmd::FToI(src, dst, value.ptr));
                                Ok(Value { ptr, ty })
                            }
                            CastDest::Float(dst) => {
                                let ptr = body.push_stack(dst_bytes);
                                body.push(Cmd::FToF(src, dst, value.ptr));
                                Ok(Value { ptr, ty })
                            }
                        }
                    }
                    _ => {
                        ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                            "Required numeric primitive for type casting",
                            format!("Found {}", value.ty.name),
                        ));
                        Err(SendError::new_error())
                    }
                }
            }
            _ => {
                ctx.push_error(ConvError::new(Severity::Error, input.span()).with_text(
                    "Required expression that returns primitive for type casting",
                    format!("Found {}", value.ty.name),
                ));
                Err(SendError::new_error())
            }
        }
    }
}
