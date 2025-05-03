use crate::prelude::*;
use ast::prim::{Float, Integer, Prim};


pub struct PrimConv;

impl Converter for PrimConv {
    type Input = Prim;
}

impl Convert<Value> for PrimConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let body = ctx.body_mut();
        body.push(Cmd::Store(input.clone()));
        match input {
            Prim::Unit => Ok(Value {
                ptr: Ptr::default(),
                ty: Ty::unit(),
            }),
            Prim::Integer(integer) => match integer {
                Integer::I8(_) => {
                    let ptr = body.push_stack(Bytes::byte());
                    Ok(Value { ptr, ty: Ty::i8() })
                }
                Integer::I16(_) => {
                    let ptr = body.push_stack(Bytes::new(2));
                    Ok(Value { ptr, ty: Ty::i16() })
                }
                Integer::I32(_) => {
                    let ptr = body.push_stack(Bytes::new(4));
                    Ok(Value { ptr, ty: Ty::i32() })
                }
                Integer::I64(_) => {
                    let ptr = body.push_stack(Bytes::new(8));
                    Ok(Value { ptr, ty: Ty::i64() })
                }
            },
            Prim::Float(float) => match float {
                Float::F32(_) => {
                    let ptr = body.push_stack(Bytes::new(4));
                    Ok(Value { ptr, ty: Ty::f32() })
                }
                Float::F64(_) => {
                    let ptr = body.push_stack(Bytes::new(8));
                    Ok(Value { ptr, ty: Ty::f64() })
                }
            },
            Prim::Bool(_) => {
                let ptr = body.push_stack(Bytes::byte());
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
            Prim::String(_) => {
                let ptr = body.push_stack(Bytes::ptr());
                Ok(Value { ptr, ty: Ty::str() })
            }
        }
    }
}
