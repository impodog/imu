use crate::prelude::*;
use ast::expr::Tuple;

// TODO: Add hint field
pub struct TupleConv;

impl Converter for TupleConv {
    type Input = Tuple;
}

impl Convert<Value> for TupleConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let mut tuple_ty = Vec::new();
        let mut size = Bytes::default();
        let mut name = String::from("(");
        let mut align = config::MEMORY_LAYOUT.init_align();
        let mut src_ptrs = Vec::new();
        for expr in input.elem.iter() {
            let value = convs::ExprConv::default()
                .convert(ctx, expr)?
                .ok_or_else(|| {
                    ctx.push_error(
                        ConvError::new(Severity::Warn, input.span())
                            .with_head("Tuple initialization requires a value"),
                    );
                    SendError::default()
                })?;

            let bytes = value.ty.size_or(input.span).map_err(|err| {
                ctx.push_error(err);
                SendError::default()
            })?;

            align = config::MEMORY_LAYOUT.update_align_by(align, bytes);
            let pad = config::memory::align_ptr_to(size, align);
            size = pad + bytes;
            name.push_str(&value.ty.name);
            name.push(',');
            tuple_ty.push(ir::sym::ty::Field {
                pad,
                item: ir::sym::ty::TyItem::Solid(value.ty.clone()),
            });
            src_ptrs.push((bytes, pad, value.ptr));
        }
        if name.chars().last().is_some_and(|ch| ch == ',') {
            name.pop().expect("There should be a comma at the end");
        }
        name.push(')');

        let body = ctx.body_mut();
        let start_ptr = body.push_stack(size);
        for (bytes, pad, ptr) in src_ptrs.into_iter() {
            body.push(Cmd::Overwrite(bytes, ptr, start_ptr + pad));
        }

        // Type is temporary here, thus there is no need to insert it into ctx
        let ty = Ty::new(ir::sym::ty::TyInner {
            name: name.into(),
            kind: ir::sym::ty::TyKind::Tuple(ir::sym::ty::Tuple(tuple_ty)),
            external: true,
        });
        Ok(Value { ptr: start_ptr, ty })
    }
}
