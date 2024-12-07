use crate::io::LineReader;
use crate::prelude::*;
use imuc_lexer::token::ResTy;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::Arc;

/// A clonable immutable handle to [`TyInner`], representing a type
///
/// This does not cause looped reference(hopefully) as the type is immutable
#[derive(Clone)]
pub struct Ty(Arc<TyInner>);
impl Deref for Ty {
    type Target = TyInner;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Ty {
    /// Creates an initial handle to the inner type
    pub fn new(inner: TyInner) -> Self {
        Self(Arc::new(inner))
    }
}

/// The inner contents of a type, containing name, sources, and memory info
#[derive(Clone)]
pub struct TyInner {
    pub name: StrRef,
    pub kind: TyKind,
    pub external: bool,
}

/// A part of [`TyInner`], holding the memory layout and features of the type
#[derive(Clone)]
pub enum TyKind {
    Res(ResTy),
    Tuple(Tuple),
    Struct(Struct),
    Ref(TyItem),
    Ptr(TyItem),
}

/// A type item included in the definition of another type
#[derive(Clone)]
pub enum TyItem {
    Solid(Ty),
    Pending(StrRef),
    Templ(crate::sym::TemplId),
}

/// A tuple type, which is an array of inner types
#[derive(Clone)]
pub struct Tuple(pub Vec<TyItem>);

/// A struct type, which is a map from names to field types
#[derive(Clone)]
pub struct Struct(pub BTreeMap<StrRef, TyItem>);

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
            _ => return Err(errors::IrError::TypeNotAllowed(format!("{:?}", self)).into()),
        };
        write!(output, "{}", str)?;
        Ok(())
    }
}

impl Rw for TyItem {
    fn read(mut input: impl IrRead) -> Result<Self> {
        let name = input.read_until(' ')?;
        if name.chars().all(|ch| ch.is_ascii_digit()) {
            let val = name.parse::<super::TemplId>()?;
            Ok(Self::Templ(val))
        } else {
            Ok(Self::Pending(name.into()))
        }
    }
    fn write(&self, mut output: impl std::io::Write) -> Result<()> {
        match self {
            Self::Solid(ty) => {
                write!(output, "{}", &*ty.name)?;
            }
            Self::Pending(name) => {
                write!(output, "{}", &**name)?;
            }
            Self::Templ(id) => {
                write!(output, "{}", id)?;
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
        match content.chars().next().ok_or_else(|| errors::IrError::Eof)? {
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
                if content.chars().rev().next().is_none_or(|ch| ch != ')') {
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
                if content.chars().rev().next().is_none_or(|ch| ch != '}') {
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
                    kind: TyKind::Struct(Struct(map)),
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
            TyKind::Struct(cus) => {
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
