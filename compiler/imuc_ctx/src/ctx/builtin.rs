use super::{Body, Ctx, Value};
use crate::prelude::*;
use cmd::Cmd;
use imuc_lexer::token::ResTy;

impl Body {
    pub fn drop_value(&mut self, value: Value) {
        let fun_name = crate::ctx::mangle::mangle_builtin_fun(&value.ty.name, ResTy::Drop);
        todo!("Glob required")
    }
}
