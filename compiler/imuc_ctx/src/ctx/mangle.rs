use super::Ctx;
use imuc_lexer::token::ResTy;

impl Ctx {
    /// Mangles a local name to a global name according to current body
    pub fn mangle_body(&self, name: &str) -> String {
        format!("{}:{}", self.body().name(), name)
    }

    /// Mangles a local name to a global name according to current body or the self body it is in
    pub fn mangle_with_self(&self, name: &str) -> String {
        if let Some(self_ty) = self.body().self_ty() {
            mangle_ty_item(self_ty.name.as_str(), name)
        } else {
            self.mangle_body(name)
        }
    }
}

/// Mangles the name of an item related to the type
pub fn mangle_ty_item(ty_name: &str, name: &str) -> String {
    format!("{}#ITM{}", ty_name, name)
}

/// Mangles the name of the ty related to a function
pub fn mangle_fun_ty(fun_name: &str) -> String {
    format!("#PAR{}", fun_name)
}

/// Mangles the name of function signature
pub fn mangle_fun_sig(fun_name: &str) -> String {
    format!("#FUN{}", fun_name)
}

/// Mangles the name of a reference
pub fn mangle_ref(name: &str) -> String {
    format!("#REF{}", name)
}

/// Mangles the name of a pointer
pub fn mangle_ptr(name: &str) -> String {
    format!("#PTR{}", name)
}

/// Mangles the name of a tupled type, such that types A, B, ... become "(A,B,...)"
pub fn tuple_name<'a, I>(tuple: I) -> String
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: Clone,
{
    // Calculates the total length of the string, and the iterator length
    let mut size = 1;
    let mut len = 0;

    let tuple = tuple.into_iter();
    for ty in tuple.clone() {
        size += ty.len() + 1;
        len += 1;
    }
    // Builds the string
    let mut s = String::with_capacity(size);
    s.push('(');
    for (index, ty) in tuple.into_iter().enumerate() {
        s.push_str(ty);
        if index + 1 != len {
            s.push(',');
        }
    }
    s.push(')');
    s
}

pub fn mangle_builtin_fun(ty: &str, res_ty: ResTy) -> String {
    let res_name = imuc_ast::builtin::associated_fun_name(res_ty);
    let name = tuple_name([res_name, ty]);
    mangle_ty_item(&name, res_name)
}
