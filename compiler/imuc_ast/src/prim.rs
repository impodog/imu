/// Syntax node of a primitive: integer, float, or string
#[derive(Clone)]
pub enum Prim {
    Bool(bool),
    Integer(Integer),
    Float(Float),
    String(String),
    Unit,
}

/// Different sizes of an integer stored in `Prim`
#[derive(Clone)]
pub enum Integer {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Any(i64),
}

impl Integer {
    /// Returns a primitive integer with size aligned to `usize`
    pub fn ptr(value: usize) -> Self {
        match std::mem::size_of::<usize>() {
            1 => Integer::I8(value as i8),
            2 => Integer::I16(value as i16),
            4 => Integer::I32(value as i32),
            8 => Integer::I64(value as i64),
            _ => unimplemented!("usize of this size is not supported"),
        }
    }
}

/// Different sizes of a float stored in `Prim`
#[derive(Clone)]
pub enum Float {
    F32(f32),
    F64(f64),
    Any(f64),
}
