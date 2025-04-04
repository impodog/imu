use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

const SMALL_STRING_THRESHOLD: usize = 60;

/// A slightly cheaper clonable reference handle to a string
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct StrRef(StrRefInner);

/// The internal enum that holds data of [`StrRef`]
///
/// The enum is two variants, namely Small and Big.
/// When the string is smaller than [`SMALL_STRING_THRESHOLD`], the Small variant is used and the
/// string is cloned completely. Otherwise the Big variant is used and the string is stored in
/// an Arc
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
enum StrRefInner {
    Small(String),
    Big(Arc<String>),
}
impl fmt::Display for StrRefInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Small(str) => fmt::Display::fmt(str, f),
            Self::Big(str) => fmt::Display::fmt(str.as_ref(), f),
        }
    }
}
impl fmt::Display for StrRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<str> for StrRef {
    fn borrow(&self) -> &str {
        self
    }
}

impl Deref for StrRefInner {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Small(str) => str.as_str(),
            Self::Big(str) => str.as_str(),
        }
    }
}
impl Deref for StrRef {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<T> for StrRefInner
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        let value: String = value.into();
        if value.len() < SMALL_STRING_THRESHOLD {
            Self::Small(value)
        } else {
            Self::Big(Arc::new(value))
        }
    }
}
impl<T> From<T> for StrRef
where
    T: Into<String>,
{
    fn from(value: T) -> Self {
        Self(StrRefInner::from(value))
    }
}
