/// Syntax node of a primitive: integer, float, or string
pub enum Prim {
    Integer(Integer),
    Float(Float),
    String(String),
    Unit,
}

/// Different sizes of an integer stored in [`Prim`]
pub enum Integer {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
}

/// Different sizes of a float stored in [`Prim`]
pub enum Float {
    F32(f32),
    F64(f64),
}
