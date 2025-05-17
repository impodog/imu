use imuc_lexer::token::ResTy;

pub static NON_FN: &str = "n/a";

pub const fn associated_fun_name(res_ty: ResTy) -> &'static str {
    #[allow(clippy::match_single_binding)]
    match res_ty {
        _ => NON_FN,
    }
}
