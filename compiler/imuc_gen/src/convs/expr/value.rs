use crate::prelude::*;
use ast::expr::Value as AstValue;
use ctx::Value;
use imuc_lexer::token::ResVal;

pub struct ValueConv;

impl Converter for ValueConv {
    type Input = AstValue;
}

impl Convert<Option<Value>> for ValueConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        let body = ctx.body_mut_or()?;
        match input {
            AstValue::Unused => Ok(None),
            AstValue::Name(name) => {
                let value = body
                    .get_value(name)
                    .ok_or_else(|| errors::ConvError::UndefinedValue(name.to_string()))?;
                Ok(Some(value.to_owned()))
            }
            AstValue::Res(res) => match res {
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
