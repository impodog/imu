use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::{LazyLock, RwLock};

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

impl StrRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// A cheaply copiable handle to a file name, however provides slower creation and querying.
/// This defaults to a name in references
pub struct Filename(usize);

static FILES: LazyLock<RwLock<Vec<StrRef>>> =
    LazyLock::new(|| RwLock::new(vec!["<reference>".into()]));

impl Filename {
    /// Creates a new file name handle
    pub fn new(value: impl Into<StrRef>) -> Self {
        // This blocks all other accesses
        let mut files = FILES.write().unwrap();
        files.push(value.into());
        Self(files.len() - 1)
    }

    /// Gets the content of this file name handle
    pub fn get(&self) -> StrRef {
        FILES
            .read()
            .unwrap()
            .get(self.0)
            .expect("created file name handles should be valid")
            .clone()
    }
}

#[derive(Debug, Clone, Copy, Default)]
/// A cursor line-column position
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, Default)]
/// A span containing file name, cursor position(left inclusive, right non-inclusive),
/// useful for error reporting
///
/// The rules for adding a span to an AST node is that when the node is specific enough to create a
/// relevant error on its own, then a span field is added
pub struct Span {
    pub file: Filename,
    pub start: Cursor,
    pub end: Cursor,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.sign_plus() {
            // FIXME: Read the file content, seek the cursor location
            todo!()
        } else {
            write!(
                f,
                "{}@[{}:{}~{}:{}]",
                self.file.get(),
                self.start.line,
                self.start.column,
                self.end.line,
                self.end.column
            )?;
        }
        Ok(())
    }
}

impl Span {
    /// Creates a referenced span that has no specific file location
    ///
    /// This is used when generating mid-process AST nodes
    pub fn refer() -> Self {
        Self {
            file: Default::default(),
            ..Default::default()
        }
    }
}
