use crate::prelude::*;
use ast::module::{Import, ImportItem, ImportItemKind};

pub struct UseConv;

impl Converter for UseConv {
    type Input = Import;
}

fn extract_alias(alias: Option<&StrRef>, name: &str) -> StrRef {
    if let Some(alias) = alias {
        alias.clone()
    } else {
        name.rfind(|ch: char| !ch.is_alphanumeric())
            .map(|x| name.get(x..).expect("Content after a rfind position"))
            .unwrap_or(name)
            .into()
    }
}

impl Convert<()> for UseConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        let cache = ctx
            .import_pool
            .write()
            .unwrap()
            .load(input.file.path())
            .map_err(|err| {
                ctx.push_error(
                    ConvError::new(Severity::Error, input.span())
                        .with_text("Error when loading module", err.to_string()),
                );
                SendError::new_error()
            })?;
        for item in input.item.iter() {
            let ImportItem { kind, alias, .. } = item;
            match kind {
                ImportItemKind::Value(name) => {
                    let alias = extract_alias(alias.as_ref(), name);
                    let fun = cache.header.fun.get(name).ok_or_else(|| {
                        ctx.push_error(ConvError::new(Severity::Error, item.span()).with_text(
                            "Import function not found in corresponding module",
                            format!("Function {name} not found"),
                        ));
                        SendError::new_error()
                    })?;
                    if !ctx.body_mut().insert_import(alias, name.clone()) {
                        ctx.push_error(ConvError::new(Severity::Warn, item.span()).with_head("Multiple imports to the same alias within one function body is not allowed. This statement has no effect"));
                    } else {
                        ctx.merge_fun([(name, fun)]);
                    }
                }
                ImportItemKind::Type(name) => {
                    let alias = extract_alias(alias.as_ref(), name);
                    let ty = cache.header.ty.get(name).ok_or_else(|| {
                        ctx.push_error(ConvError::new(Severity::Error, item.span()).with_text(
                            "Import type not found in corresponding module",
                            format!("Type {name} not found"),
                        ));
                        SendError::new_error()
                    })?;
                    if !ctx.body_mut().insert_import(alias, name.clone()) {
                        ctx.push_error(ConvError::new(Severity::Warn, item.span()).with_head("Multiple imports to the same alias within one function body is not allowed. This statement has no effect"));
                    } else {
                        ctx.body_mut().locals_mut().ty.insert(ty.clone());
                    }
                }
            }
        }
        Ok(())
    }
}
