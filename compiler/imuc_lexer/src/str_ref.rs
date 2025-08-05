use std::borrow::Borrow;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::{LazyLock, RwLock};

const SMALL_STRING_THRESHOLD: usize = 60;

/// A slightly cheaper clonable reference handle to a string
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct StrRef(StrRefInner);

static EMPTY_STR_REF: LazyLock<StrRef> = LazyLock::new(|| StrRef::from(""));

impl Default for StrRef {
    fn default() -> Self {
        EMPTY_STR_REF.clone()
    }
}

/// The internal enum that holds data of `StrRef`
///
/// The enum is two variants, namely Small and Big.
/// When the string is smaller than `SMALL_STRING_THRESHOLD`, the Small variant is used and the
/// string is cloned completely. Otherwise the Big variant is used and the string is stored in
/// an Arc
#[derive(Clone, Debug)]
enum StrRefInner {
    Small(String),
    Big(Arc<String>),
}
impl StrRefInner {
    fn as_str(&self) -> &str {
        self
    }
}

// Custom implementations of std traits
impl std::cmp::PartialOrd for StrRefInner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl std::cmp::Ord for StrRefInner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}
impl std::cmp::PartialEq for StrRefInner {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq(other.as_str())
    }
}
impl std::cmp::Eq for StrRefInner {}
impl std::hash::Hash for StrRefInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
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
        let mut value: String = value.into();
        value.shrink_to_fit();
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
    pub fn into_string(self) -> String {
        match self.0 {
            StrRefInner::Small(string) => string,
            StrRefInner::Big(arc) => arc.to_string(),
        }
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

impl Cursor {
    pub fn with_column(self, column: usize) -> Self {
        Self {
            line: self.line,
            column,
        }
    }
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
        write!(
            f,
            "{}@[{}:{}~{}:{}]",
            self.file.get(),
            self.start.line,
            self.start.column,
            self.end.line,
            self.end.column
        )?;
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
