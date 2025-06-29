use imuc_ast::name::Prefix;

use crate::prelude::*;

/// Converts the alias to its real name, if any
pub(crate) fn resolve_alias(ctx: &mut Ctx, name: &StrRef) -> StrRef {
    if let Some(real_name) = ctx.body_mut().get_import(name.as_str()) {
        real_name
    } else {
        name.clone()
    }
}

/// Converts a prefixed item(possibly with aliases) into its real location name
pub struct PrefixConv<'prefix> {
    pub prefix: &'prefix Prefix,
}

impl<'prefix> Converter for PrefixConv<'prefix> {
    type Input = StrRef;
}

impl<'prefix> Convert<String> for PrefixConv<'prefix> {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<String> {
        let base = match &self.prefix.first {
            ast::name::PrefixFirst::Name(name) => resolve_alias(ctx, name),
            ast::name::PrefixFirst::Loc => ctx.base_name().into(),
        };
        let mut base = base.into_string();
        // Prefix iterator starts after the first element of the prefix,
        for next in self.prefix.iter() {
            base = ctx::mangle::mangle_inside_body(base.as_str(), next.as_str());
        }
        Ok(ctx::mangle::mangle_body_item(base.as_str(), input.as_str()))
    }
}
