use crate::prelude::*;
use ast::module::{Import, ImportItem, ImportItemKind};
use imuc_ast::module::ImportDir;
use imuc_ctx::ctx::ImportCache;

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

fn fun_is_public(cache: &ImportCache, name: &str) -> bool {
    let fun_ty_name = ctx::mangle::mangle_fun_sig(name);
    cache.header.ty.contains_key(fun_ty_name.as_str())
}

impl Convert<()> for UseConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        match input.dir {
            ImportDir::External(ref dir) => {
                let cache = {
                    let import_pool = ctx.import_pool.clone();
                    let cache = import_pool.write().unwrap().load(ctx, dir).map_err(|err| {
                        ctx.push_error(
                            ConvError::new(Severity::Error, input.span())
                                .with_text(format!("When loading module {dir:?}"), err.to_string()),
                        );
                        SendError::new_error()
                    })?;
                    cache
                };
                for item in input.item.iter() {
                    let ImportItem {
                        kind,
                        alias,
                        prefix,
                        ..
                    } = item;
                    match kind {
                        ImportItemKind::Value(name) => {
                            let alias = extract_alias(alias.as_ref(), name);
                            let name =
                                StrRef::from(convs::PrefixConv { prefix }.convert(ctx, name)?);

                            // Check whether it is a private function by checking whether the
                            // function signature type is exported
                            if !fun_is_public(cache.as_ref(), name.as_ref()) {
                                ctx.push_error(
                                    ConvError::new(Severity::Error, item.span()).with_text(
                                        format!(
                                            "Import function is present in {dir:?}, but is private"
                                        ),
                                        format!("Function {name} is private"),
                                    ),
                                );
                                return Err(SendError::new_error());
                            }

                            let fun = cache.header.fun.get(&name).ok_or_else(|| {
                                ctx.push_error(
                                    ConvError::new(Severity::Error, item.span()).with_text(
                                        "Import function not found in corresponding module",
                                        format!("Function {name} not found"),
                                    ),
                                );
                                SendError::new_error()
                            })?;
                            if !ctx.body_mut().insert_import(alias, name.clone()) {
                                ctx.push_error(ConvError::new(Severity::Warn, item.span()).with_head("Multiple imports to the same alias within one body is not allowed. This statement has no effect"));
                            } else {
                                ctx.merge_fun([(&name, fun)]);
                            }
                        }
                        ImportItemKind::Type(name) => {
                            let alias = extract_alias(alias.as_ref(), name);
                            let name =
                                StrRef::from(convs::PrefixConv { prefix }.convert(ctx, name)?);
                            let ty = cache.header.ty.get(&name).ok_or_else(|| {
                                ctx.push_error(
                                    ConvError::new(Severity::Error, item.span()).with_text(
                                        "Import type not found in corresponding module",
                                        format!("Type {name} not found"),
                                    ),
                                );
                                SendError::new_error()
                            })?;
                            if !ctx.body_mut().insert_import(alias, name.clone()) {
                                ctx.push_error(ConvError::new(Severity::Warn, item.span()).with_head("Multiple imports to the same alias within one body is not allowed. This statement has no effect"));
                            } else {
                                ctx.body_mut().locals_mut().ty.insert(ty.clone());
                            }
                        }
                        // NOTE: For wildcards, alias/prefix after first is ignored because parser didn't handle alias/prefix after first
                        // and will see this extra part as a syntax error
                        ImportItemKind::WildcardWithPrefix => {
                            ctx.merge_fun(
                                cache.header.fun.iter().filter(|(name, _)| {
                                    fun_is_public(cache.as_ref(), name.as_str())
                                }),
                            );
                            // Ty is previously imported by TyMap::merge
                        }
                        ImportItemKind::Wildcard => {
                            ctx.merge_fun(
                                cache.header.fun.iter().filter(|(name, _)| {
                                    fun_is_public(cache.as_ref(), name.as_str())
                                }),
                            );
                            for name in cache.header.fun.keys() {
                                if fun_is_public(cache.as_ref(), name.as_str()) {
                                    if let Some(alias) = ctx::mangle::extract_name_last_part(name) {
                                        ctx.body_mut().insert_import(alias.into(), name.clone());
                                    }
                                }
                            }
                            for name in cache.header.ty.keys() {
                                if let Some(alias) = ctx::mangle::extract_name_last_part(name) {
                                    ctx.body_mut().insert_import(alias.into(), name.clone());
                                }
                            }
                        }
                    }
                }
            }
            ImportDir::Loc => {
                todo!("Local alias")
            }
        }
        Ok(())
    }
}
