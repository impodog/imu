use crate::cmd::Bytes;
use crate::io::LineReader;
use crate::prelude::*;
use imuc_lexer::token::ResTy;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::OnceLock;

/// A clonable immutable handle to [`TyInner`], representing a type
///
/// This should not cause looped reference when creating
#[derive(Clone)]
pub struct Ty(Arc<(TyInner, OnceLock<Option<Bytes>>)>);
impl Deref for Ty {
    type Target = TyInner;
    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

macro_rules! generate_reserved {
    ($func: ident, $name: literal, $ty: ident) => {
        pub fn $func() -> Self {
            static VALUE: OnceLock<Ty> = OnceLock::new();

            VALUE
                .get_or_init(|| {
                    Ty::new(TyInner {
                        name: $name.into(),
                        kind: TyKind::Res(ResTy::$ty),
                        external: true,
                    })
                })
                .clone()
        }
    };
}

impl Ty {
    /// Creates an initial handle to the inner type
    pub fn new(inner: TyInner) -> Self {
        Self(Arc::new((inner, OnceLock::new())))
    }

    /// Tests if the types are same-by-name
    pub fn test_eq(&self, ty: &Ty) -> bool {
        self.size() == ty.size() && self.name == ty.name
    }

    /// Converts the type to [`ResTy`], if possible
    pub fn to_res_ty(&self) -> Option<ResTy> {
        match &self.0 .0.kind {
            TyKind::Res(res_ty) => Some(*res_ty),
            _ => None,
        }
    }
    /// Builds a ty from [`ResTy`], if possible
    pub fn from_res(res_ty: ResTy) -> Option<Self> {
        let ty = match res_ty {
            ResTy::SelfType => return None,
            ResTy::Unit => Self::unit(),
            ResTy::Bool => Self::bool(),
            ResTy::I8 => Self::i8(),
            ResTy::I16 => Self::i16(),
            ResTy::I32 => Self::i32(),
            ResTy::I64 => Self::i64(),
            ResTy::F32 => Self::f32(),
            ResTy::F64 => Self::f64(),
            ResTy::Ptr => Self::ptr(),
            ResTy::Str => Self::str(),
            ResTy::Drop => Self::drop(),
        };
        Some(ty)
    }

    /// Calculates the size of the type in bytes, and store it for future use
    ///
    /// If the type contains unresolved types or ResTy::SelfType or TyKind::Fun, [`None`] is returned
    pub fn size(&self) -> Option<Bytes> {
        self.0
             .1
            .get_or_init(|| {
                let len = match &self.0 .0.kind {
                    TyKind::Res(res) => {
                        let len = match res {
                            ResTy::SelfType => return None,
                            ResTy::Unit => 0,
                            ResTy::I8 | ResTy::Bool => 1,
                            ResTy::I16 => 2,
                            ResTy::I32 | ResTy::F32 => 4,
                            ResTy::I64 | ResTy::F64 => 8,
                            ResTy::Str | ResTy::Ptr => crate::cmd::PTR_SIZE,
                            ResTy::Drop => 0,
                        };
                        Bytes::new(len)
                    }
                    TyKind::Ptr(_) | TyKind::Ref(_) => Bytes::ptr(),
                    TyKind::Fun { .. } => return None,
                    TyKind::Tuple(tuple) => {
                        let mut accum = Bytes::default();
                        for value in tuple.0.iter() {
                            accum += value.size()?;
                        }
                        accum
                    }
                    TyKind::Cus(cus) => {
                        let mut accum = Bytes::default();
                        for value in cus.0.values() {
                            accum += value.size()?;
                        }
                        accum
                    }
                };
                Some(len)
            })
            .as_ref()
            .copied()
    }

    generate_reserved!(unit, "Unit", Unit);
    generate_reserved!(bool, "Bool", Bool);
    generate_reserved!(i8, "I8", I8);
    generate_reserved!(i16, "I16", I16);
    generate_reserved!(i32, "I32", I32);
    generate_reserved!(i64, "I64", I64);
    generate_reserved!(f32, "F32", F32);
    generate_reserved!(f64, "F64", F64);
    generate_reserved!(ptr, "Ptr", Ptr);
    generate_reserved!(str, "Str", Str);
    generate_reserved!(drop, "Drop", Drop);
}

/// The inner contents of a type, containing name, sources, and memory info
#[derive(Clone)]
pub struct TyInner {
    pub name: StrRef,
    pub kind: TyKind,
    pub external: bool,
}

impl TyInner {
    /// Creates a local type (possibly generated from user code)
    pub fn new(name: StrRef, kind: TyKind) -> Self {
        Self {
            name,
            kind,
            external: false,
        }
    }
}

/// A part of [`TyInner`], holding the memory layout and features of the type
#[derive(Clone)]
pub enum TyKind {
    Res(ResTy),
    Fun { param: TyItem, ret: TyItem },
    Tuple(Tuple),
    Cus(Cus),
    Ref(TyItem),
    Ptr(TyItem),
}

/// A type item included in the definition of another type
#[derive(Clone)]
pub enum TyItem {
    Solid(Ty),
    Pending(StrRef),
}

impl TyItem {
    /// Gets the size of the type in bytes, or None if the type is uninitialized
    pub fn size(&self) -> Option<Bytes> {
        match self {
            Self::Solid(ty) => ty.size(),
            _ => None,
        }
    }

    /// Gets the name of this [`TyItem`]
    pub fn name(&self) -> &StrRef {
        match self {
            Self::Solid(ty) => &ty.name,
            Self::Pending(name) => name,
        }
    }
}

/// A tuple type, which is an array of inner types
#[derive(Clone)]
pub struct Tuple(pub Vec<TyItem>);

/// A struct type, which is a map from names to field types
#[derive(Clone)]
pub struct Cus(pub BTreeMap<StrRef, TyItem>);

impl Rw for ResTy {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = input.read_until(' ')?;
        let res = match name {
            "I8" => ResTy::I8,
            "I16" => ResTy::I16,
            "I32" => ResTy::I32,
            "I64" => ResTy::I64,
            "F32" => ResTy::F32,
            "F64" => ResTy::F64,
            "Str" => ResTy::Str,
            "Ptr" => ResTy::Ptr,
            "Drop" => ResTy::Drop,
            _ => return Err(errors::IrError::NoSuchType(name.to_owned()).into()),
        };
        Ok(res)
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        let str = match self {
            ResTy::I8 => "I8",
            ResTy::I16 => "I16",
            ResTy::I32 => "I32",
            ResTy::I64 => "I64",
            ResTy::F32 => "F32",
            ResTy::F64 => "F64",
            ResTy::Str => "Str",
            ResTy::Ptr => "Ptr",
            ResTy::Drop => "Drop",
            _ => return Err(errors::IrError::TypeNotAllowed(format!("{:?}", self)).into()),
        };
        write!(output, "{}", str)?;
        Ok(())
    }
}

impl From<Ty> for TyItem {
    fn from(value: Ty) -> Self {
        TyItem::Solid(value)
    }
}

impl Rw for TyItem {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = input.read_until(' ')?;
        Ok(Self::Pending(name.into()))
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        match self {
            Self::Solid(ty) => {
                write!(output, "{}", &*ty.name)?;
            }
            Self::Pending(name) => {
                write!(output, "{}", &**name)?;
            }
        }
        Ok(())
    }
}

impl Rw for Ty {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = StrRef::from(input.read_until(' ')?);
        let external = input.external();
        let content = input.read_line()?;
        match content.chars().next().ok_or(errors::IrError::Eof)? {
            ',' => {
                let mut reader = LineReader::new(&content[1..], external);
                let param = TyItem::read(&mut reader)?;
                let ret = TyItem::read(&mut reader)?;
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Fun { param, ret },
                    external: input.external(),
                }))
            }
            '&' => {
                let item = TyItem::read(LineReader::new(&content[1..], external))?;
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Ref(item),
                    external: input.external(),
                }))
            }
            '@' => {
                let item = TyItem::read(LineReader::new(&content[1..], external))?;
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Ptr(item),
                    external: input.external(),
                }))
            }
            '(' => {
                if content.chars().next_back().is_none_or(|ch| ch != ')') {
                    return Err(errors::IrError::Unmatched('(', ')').into());
                }
                let values = content[1..content.len() - 1].split(',');
                let mut tuple = Vec::new();
                for value in values {
                    let item = TyItem::read(LineReader::new(value, external))?;
                    tuple.push(item);
                }
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Tuple(Tuple(tuple)),
                    external: input.external(),
                }))
            }
            '{' => {
                if content.chars().next_back().is_none_or(|ch| ch != '}') {
                    return Err(errors::IrError::Unmatched('{', '}').into());
                }
                let values = content[1..content.len() - 1].split(',');
                let mut map = BTreeMap::new();
                for value in values {
                    let ok = if let Some(colon) = value.find(':') {
                        if colon + 1 != value.len() {
                            let name = StrRef::from(&value[..colon]);
                            let item =
                                TyItem::read(LineReader::new(&value[colon + 1..], external))?;
                            map.insert(name, item);
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !ok {
                        return Err(errors::IrError::CharRequired(':').into());
                    }
                }
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Cus(Cus(map)),
                    external: input.external(),
                }))
            }
            _ => {
                let res = ResTy::read(LineReader::new(content, external))?;
                // TODO: Reuse reserved type definitions to save space
                Ok(Ty::new(TyInner {
                    name,
                    kind: TyKind::Res(res),
                    external: input.external(),
                }))
            }
        }
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        if self.external {
            return Err(errors::IrError::InternalRequired.into());
        }
        write!(output, "{} ", &*self.name)?;
        match &self.kind {
            TyKind::Res(res) => (*res).write(output)?,
            TyKind::Fun { param, ret } => {
                write!(output, "! ")?;
                param.write(&mut output)?;
                write!(output, " ")?;
                ret.write(&mut output)?;
            }
            TyKind::Ref(ty) => {
                write!(output, "&")?;
                ty.write(output)?;
            }
            TyKind::Ptr(ty) => {
                write!(output, "@")?;
                ty.write(output)?;
            }
            TyKind::Tuple(tuple) => {
                write!(output, "(")?;
                for (is_last, ty) in tuple
                    .0
                    .iter()
                    .enumerate()
                    .map(|(i, ty)| (i + 1 == tuple.0.len(), ty))
                {
                    ty.write(&mut output)?;
                    if !is_last {
                        write!(output, ",")?;
                    }
                }
                write!(output, ")")?;
            }
            TyKind::Cus(cus) => {
                write!(output, "{{")?;
                for (is_last, (name, ty)) in cus
                    .0
                    .iter()
                    .enumerate()
                    .map(|(i, value)| (i + 1 == cus.0.len(), value))
                {
                    write!(output, "{}=", &**name)?;
                    ty.write(&mut output)?;
                    if !is_last {
                        write!(output, ",")?;
                    }
                }
                write!(output, "}}")?;
            }
        }
        Ok(())
    }
}
