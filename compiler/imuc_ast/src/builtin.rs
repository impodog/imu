use imuc_lexer::token::ResTy;

pub static DROP_FN: &str = "drop";
pub static NON_FN: &str = "n/a";

pub const fn associated_fun_name(res_ty: ResTy) -> &'static str {
    match res_ty {
        ResTy::Drop => DROP_FN,
        _ => NON_FN,
    }
}
