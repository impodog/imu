use crate::prelude::*;
use ast::expr::Value as AstValue;
use ast::expr::ValueInner;

use imuc_lexer::token::ResVal;

pub struct ValueConv;

impl Converter for ValueConv {
    type Input = AstValue;
}

impl Convert<Option<Value>> for ValueConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        let body = ctx.body_mut();
        match &input.value {
            ValueInner::Unused => Ok(None),
            ValueInner::Name(name) => {
                if let Some(value) = body.get_value(name.as_str()) {
                    Ok(Some(value.to_owned()))
                } else if let Some(value) = body
                    .get_import(name.as_str())
                    .and_then(|name| body.get_value(name.as_str()))
                {
                    Ok(Some(value.to_owned()))
                } else {
                    let globs = body.globs.clone();
                    let globs_lock = globs.read().unwrap();
                    if let Some(glob) = globs_lock.get(name.as_str()) {
                        let ptr = body.push_stack(Bytes::ptr());
                        body.push(Cmd::StorePtr(glob.ptr()));
                        Ok(Some(Value {
                            ptr,
                            ty: glob.ty().clone(),
                        }))
                    } else {
                        ctx.push_error(
                            ConvError::new(Severity::Error, input.span()).with_text(
                                "Undefined name",
                                format!("Value {} is undefined", name),
                            ),
                        );
                        Err(SendError::new_error())
                    }
                }
            }
            ValueInner::Res(res) => match res {
                ResVal::True => {
                    let ptr = body.push_stack(Bytes::byte());
                    body.push(Cmd::Store(ast::prim::Prim::Bool(true)));
                    Ok(Some(Value {
                        ptr,
                        ty: ir::sym::Ty::bool(),
                    }))
                }
                ResVal::False => {
                    let ptr = body.push_stack(Bytes::byte());
                    body.push(Cmd::Store(ast::prim::Prim::Bool(false)));
                    Ok(Some(Value {
                        ptr,
                        ty: ir::sym::Ty::bool(),
                    }))
                }
            },
        }
    }
}
