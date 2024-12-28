use crate::prelude::*;
use ast::expr::Value;
use imuc_lexer::token::ResVal;

pub struct ValueConv;

impl Converter for ValueConv {
    type Input = Value;
}

impl Convert<Option<Ptr>> for ValueConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Ptr>> {
        match input {
            Value::Unused => Ok(None),
            Value::Name(name) => {
                let value = ctx
                    .get_value(name)
                    .ok_or_else(|| errors::ConvError::UndefinedValue(name.to_string()))?;
                Ok(Some(value.ptr))
            }
            Value::Res(res) => match res {
                ResVal::True => {
                    let ptr = ctx.push_stack(Bytes::new(0));
                    ctx.body_mut_or()?
                        .push(Cmd::Store(ast::prim::Prim::Bool(true)));
                    Ok(Some(ptr))
                }
                ResVal::False => {
                    let ptr = ctx.push_stack(Bytes::new(0));
                    ctx.body_mut_or()?
                        .push(Cmd::Store(ast::prim::Prim::Bool(false)));
                    Ok(Some(ptr))
                }
            },
        }
    }
}
