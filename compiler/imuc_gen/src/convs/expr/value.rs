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
        let error_queue = ctx.error_queue.clone();
        let body = ctx.body_mut();
        match &input.value {
            ValueInner::Unused => Ok(None),
            ValueInner::Name(name) => {
                let value = body.get_value(name.as_str()).ok_or_else(|| {
                    error_queue.write().unwrap().push_back(
                        ConvError::new(Severity::Error, input.span())
                            .with_text("Undefined value", format!("Name {} not found", name))
                            .into(),
                    );
                    SendError::default()
                })?;
                Ok(Some(value.to_owned()))
            }
            ValueInner::Res(res) => match res {
                ResVal::True => {
                    let ptr = body.push_stack(Bytes::new(1));
                    body.push(Cmd::Store(ast::prim::Prim::Bool(true)));
                    Ok(Some(Value {
                        ptr,
                        ty: ir::sym::Ty::bool(),
                    }))
                }
                ResVal::False => {
                    let ptr = body.push_stack(Bytes::new(1));
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
