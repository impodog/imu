use super::{Body, Value};
use crate::prelude::*;
use cmd::{Bytes, Cmd};
use imuc_lexer::token::ResTy;
use sym::Ty;

impl Body {
    pub fn drop_value(&mut self, value: &Value) -> Result<()> {
        if value.ty.test_eq(&Ty::unit()) {
            // No action should be performed with unit type
            Ok(())
        } else {
            let fun_name = crate::ctx::mangle::mangle_builtin_fun(&value.ty.name, ResTy::Drop);
            self.with_globs(move |body, globs| -> Result<()> {
                if let Some(glob) = globs.get(fun_name.as_str()) {
                    body.push(Cmd::StorePtr(glob.ptr()));
                    body.push_stack(Bytes::ptr());
                    // FIXME: A memory move is performed. Fix?
                    let size = value.ty.size_or()?;
                    body.push(Cmd::Dupli(size, value.ptr));
                    body.push(Cmd::Call(size));
                } else {
                    todo!("There is no drop implementation for type {}", value.ty.name);
                }
                Ok(())
            })
        }
    }
}
