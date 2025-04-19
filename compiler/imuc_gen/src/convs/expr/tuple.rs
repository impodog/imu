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
            let body = ctx.body_mut();
            body.push(Cmd::Dupli(value.ptr, bytes));

            size += bytes;
            name.push_str(&value.ty.name);
            name.push(',');
            tuple_ty.push(ir::sym::ty::TyItem::Solid(value.ty.clone()));
        }
        name.push(')');

        let ptr = ctx.body_mut().push_stack(size);
        // Type is temporary here, thus there is no need to insert it into ctx
        let ty = Ty::new(ir::sym::ty::TyInner {
            name: name.into(),
            kind: ir::sym::ty::TyKind::Tuple(ir::sym::ty::Tuple(tuple_ty)),
            external: true,
        });
        Ok(Value { ptr, ty })
    }
}
