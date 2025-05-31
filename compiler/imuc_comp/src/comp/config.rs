use crate::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct Requirement {
    pub name: String,
    #[serde(default)]
    pub path: Option<PathBuf>,
}

fn lib() -> PathBuf {
    "lib.iu".into()
}
fn target() -> PathBuf {
    "target".into()
}

#[derive(Default, Serialize, Deserialize)]
pub struct Target {
    #[serde(default = "lib")]
    pub lib: PathBuf,
    #[serde(default = "target")]
    pub output_dir: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Comp {
    #[serde(default)]
    pub req: Vec<Requirement>,
    pub target: Target,
}

impl Comp {
    pub fn read_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config = toml::from_str(content.as_str())?;
        Ok(config)
    }
}
