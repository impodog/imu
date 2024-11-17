mod file;
mod module;
mod prelude;
mod resolve;

pub use file::File;
pub use module::{Module, SubModule};
pub use resolve::Resolver;

pub const SUFFIX: &'static str = "iu";
