use crate::prelude::*;
use ast::bind::{Bind, Let};
use ast::pat::{IdentKind, IdentPat, Pat, TuplePat, Type};

pub struct BindConv;

impl Converter for BindConv {
    type Input = Bind;
}

impl Convert<()> for BindConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<()> {
        match input {
            Bind::Let(bind) => {
                let Let { pat, val } = bind;
                match pat {
                    Pat::Ident(ident) => {
                        let ty = if let Some(ref ty) = ident.ty {
                            convs::TypeConv.convert(ctx, ty)?
                        } else {
                            None
                        };
                        let val = convs::ExprConv::default().with_hint(ty).convert(ctx, val)?;
                        match &ident.ident {
                            IdentKind::Unused => Ok(()),
                            IdentKind::Value(name) => {
                                // TODO: Maybe drop variables with the same name before inserting
                                todo!()
                            }
                        }
                    }
                    _ => todo!(),
                }
            }
            Bind::Item(item) => {
                todo!()
            }
        }
    }
}
