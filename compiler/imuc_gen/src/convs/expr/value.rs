use crate::prelude::*;
use ast::expr::Value as AstValue;
use ast::expr::ValueInner;

use imuc_ir::sym::ty::TyKind;
use imuc_lexer::token::ResVal;

fn get_glob(ctx: &mut Ctx, name: &str) -> Option<Value> {
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

fn resolve_function(ctx: &mut Ctx, head: &Value, nest: &StrRef) -> Option<Value> {
    let item = ctx::mangle::mangle_ty_item(&head.ty.name, nest);
    get_glob(ctx, item.as_str())
}

fn resolve_member(
    ctx: &mut Ctx,
    head: &Value,
    nest: &StrRef,
    span: imuc_lexer::Span,
) -> Result<Option<Value>> {
    // NOTE: This function only returns Err because of type resolution, which is not possible in a
    // normal parsing file
    match &head.ty.kind {
        TyKind::Cus(cus) => {
            // FIXME: Anyway to prevent iterating?
            let mut ptr = Ptr::default();
            for (name, ty) in cus.0.iter() {
                if name == nest {
                    return Ok(Some(Value {
                        ptr: head.ptr + ptr,
                        ty: ctx
                            .ty
                            .resolve_or(ty, span)
                            .map_err(|err| {
                                ctx.push_error(err);
                                SendError::new_error()
                            })?
                            .clone(),
                    }));
                }
                ptr += ty.size().ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(Severity::Fatal, span)
                            .with_head("Resolved type required in member resolution"),
                    );
                    SendError::new_error()
                })?;
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[allow(clippy::collapsible_else_if)]
fn resolve_nest(
    ctx: &mut Ctx,
    mut head: Value,
    tail: &[StrRef],
    span: imuc_lexer::Span,
) -> Result<Value> {
    let mut is_self = ctx
        .body()
        .self_ty()
        .is_some_and(|self_ty| self_ty.test_eq(&head.ty));
    for nest in tail.iter() {
        let resolved = if is_self {
            if let Some(value) = resolve_member(ctx, &head, nest, span)? {
                head = value;
                true
            } else if let Some(value) = resolve_function(ctx, &head, nest) {
                head = value;
                true
            } else {
                false
            }
        } else {
            if let Some(value) = resolve_function(ctx, &head, nest) {
                head = value;
                true
            } else if let Some(value) = resolve_member(ctx, &head, nest, span)? {
                head = value;
                true
            } else {
                false
            }
        };
        if !resolved {
            ctx.push_error(ConvError::new(Severity::Error, span).with_text(
                "Unable to resolve member or function",
                format!("Unable to resolve {} of type {}", nest, head.ty.name),
            ));
            return Err(SendError::new_error());
        }
        // This removes member-first resolving for deeper nests, because they are not self
        is_self = false;
    }
    Ok(head)
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
            ValueInner::Nested(list) => {
                let nonempty::NonEmpty { head, tail } = list;
                let head = search_name(ctx, head, None, input.span())?;
                resolve_nest(ctx, head, tail, input.span()).map(Some)
            }
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
