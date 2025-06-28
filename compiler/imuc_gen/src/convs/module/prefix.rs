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
        let mut base = resolve_alias(ctx, self.prefix.first()).into_string();
        for next in self.prefix.iter().skip(1) {
            base = ctx::mangle::mangle_inside_body(base.as_str(), next.as_str());
        }
        Ok(ctx::mangle::mangle_body_item(base.as_str(), input.as_str()))
    }
}
