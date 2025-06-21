use crate::prelude::*;
use ast::expr::Value as AstValue;
use ast::expr::ValueInner;

use imuc_lexer::token::ResVal;

pub(crate) fn get_glob(ctx: &mut Ctx, name: &str) -> Option<Value> {
    let body = ctx.body_mut();
    let globs = body.globs.clone();
    let globs_lock = globs.read().unwrap();
    if let Some(glob) = globs_lock.get(name) {
        let ptr = body.push_stack(Bytes::ptr());
        body.push(Cmd::StorePtr(glob.ptr()));
        Some(Value {
            ptr,
            ty: glob.ty().clone(),
        })
    } else {
        None
    }
}

fn search_name(
    ctx: &mut Ctx,
    name: &StrRef,
    alias: Option<&StrRef>,
    span: imuc_lexer::Span,
) -> Result<Value> {
    if alias.is_none() {
        if let Some(target_name) = ctx.body_mut().get_import(name.as_str()) {
            return search_name(ctx, &target_name, Some(name), span);
        }
    }
    if let Some(value) = ctx.body_mut().get_value(name.as_str()) {
        return Ok(value.to_owned());
    }
    if let Some(value) = get_glob(ctx, name.as_str()) {
        return Ok(value);
    }

    ctx.push_error(ConvError::new(Severity::Error, span).with_text(
        "Undefined name",
        if let Some(alias) = alias {
            format!("Value {} (alias {}) is undefined", name, alias)
        } else {
            format!("Value {} is undefined", name)
        },
    ));
    Err(SendError::new_error())
}

pub struct ValueConv;

impl Converter for ValueConv {
    type Input = AstValue;
}

impl Convert<Option<Value>> for ValueConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Option<Value>> {
        let body = ctx.body_mut();
        match &input.value {
            ValueInner::Unused => Ok(None),
            ValueInner::Name(name) => search_name(ctx, name, None, input.span()).map(Some),
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
