use super::ExprSolver;
use crate::prelude::*;
use ast::prim::{Float, Integer, Prim};
use imuc_lexer::token::ResTy;

pub struct PrimConv {
    pub solver: ExprSolver,
}

impl Converter for PrimConv {
    type Input = Prim;
}

fn convert_integer(body: &mut ctx::Body, solver: ExprSolver, integer: Integer) -> Result<Value> {
    match integer {
        Integer::I8(_) => {
            let ptr = body.push_cmd(Bytes::byte(), Cmd::Store(Prim::Integer(integer)));
            Ok(Value { ptr, ty: Ty::i8() })
        }
        Integer::I16(_) => {
            let ptr = body.push_cmd(Bytes::new(2), Cmd::Store(Prim::Integer(integer)));
            Ok(Value { ptr, ty: Ty::i16() })
        }
        Integer::I32(_) => {
            let ptr = body.push_cmd(Bytes::new(4), Cmd::Store(Prim::Integer(integer)));
            Ok(Value { ptr, ty: Ty::i32() })
        }
        Integer::I64(_) => {
            let ptr = body.push_cmd(Bytes::new(8), Cmd::Store(Prim::Integer(integer)));
            Ok(Value { ptr, ty: Ty::i64() })
        }
        Integer::Any(value) => {
            if let Some(hint) = &solver.hint {
                match hint.kind {
                    ir::sym::ty::TyKind::Res(res) => match res {
                        ResTy::I8 => convert_integer(body, solver, Integer::I8(value as i8)),
                        ResTy::I16 => convert_integer(body, solver, Integer::I16(value as i16)),
                        ResTy::I32 => convert_integer(body, solver, Integer::I32(value as i32)),
                        ResTy::I64 => convert_integer(body, solver, Integer::I64(value)),
                        _ => convert_integer(body, solver, Integer::I32(value as i32)),
                    },
                    _ => convert_integer(body, solver, Integer::I32(value as i32)),
                }
            } else {
                convert_integer(body, solver, Integer::I32(value as i32))
            }
        }
    }
}

fn convert_float(body: &mut ctx::Body, solver: ExprSolver, float: Float) -> Result<Value> {
    match float {
        Float::F32(_) => {
            let ptr = body.push_cmd(Bytes::new(4), Cmd::Store(Prim::Float(float)));
            Ok(Value { ptr, ty: Ty::f32() })
        }
        Float::F64(_) => {
            let ptr = body.push_cmd(Bytes::new(8), Cmd::Store(Prim::Float(float)));
            Ok(Value { ptr, ty: Ty::f64() })
        }
        Float::Any(value) => {
            if let Some(hint) = &solver.hint {
                match hint.kind {
                    ir::sym::ty::TyKind::Res(res) => match res {
                        ResTy::F32 => convert_float(body, solver, Float::F32(value as f32)),
                        ResTy::F64 => convert_float(body, solver, Float::F64(value)),
                        _ => convert_float(body, solver, Float::F32(value as f32)),
                    },
                    _ => convert_float(body, solver, Float::F32(value as f32)),
                }
            } else {
                convert_float(body, solver, Float::F32(value as f32))
            }
        }
    }
}

impl Convert<Value> for PrimConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        let Self { solver } = self;
        let body = ctx.body_mut();
        match input {
            Prim::Unit => {
                // NOTE: Not using `push_void` is intentional, to align the pointers even on a unit
                // type
                body.push_cmd(Bytes::null(), Cmd::Store(input.clone()));
                Ok(Value {
                    ptr: Ptr::default(),
                    ty: Ty::unit(),
                })
            }
            Prim::Integer(integer) => convert_integer(body, solver, integer.clone()),
            Prim::Float(float) => convert_float(body, solver, float.clone()),
            Prim::Bool(_) => {
                let ptr = body.push_cmd(Bytes::byte(), Cmd::Store(input.clone()));
                Ok(Value {
                    ptr,
                    ty: Ty::bool(),
                })
            }
            Prim::String(_) => {
                let ptr = body.push_cmd(Bytes::ptr(), Cmd::Store(input.clone()));
                Ok(Value { ptr, ty: Ty::str() })
            }
        }
    }
}
