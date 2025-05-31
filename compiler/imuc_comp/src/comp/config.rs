use crate::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Default, Serialize, Deserialize)]
pub struct Requirement {
    pub name: String,
    #[serde(default)]
    pub path: Option<PathBuf>,
}

fn root() -> PathBuf {
    "lib.iu".into()
}
fn output() -> PathBuf {
    "target".into()
}

#[derive(Default, Serialize, Deserialize)]
pub struct Target {
    pub module: String,
    #[serde(default = "root")]
    pub root: PathBuf,
    #[serde(default = "output")]
    pub output: PathBuf,
}

#[derive(Default, Serialize, Deserialize)]
pub struct Env {
    pub cwd: PathBuf,
}

impl Env {
    /// Changes the path according to the module cwd to the compiler program cwd,
    /// if the path is relative
    pub fn change_cwd(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.cwd.join(path)
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Comp {
    #[serde(default)]
    pub req: Vec<Requirement>,
    pub target: Target,
    #[serde(default)]
    pub env: Env,
}

#[derive(Debug, Clone, Copy)]
struct NotAFile;

impl std::fmt::Display for NotAFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Given config path is not a file")
    }
}

impl std::error::Error for NotAFile {}

impl Comp {
    pub fn read_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Comp = toml::from_str(content.as_str())?;
        if config.env.cwd.as_os_str().is_empty() {
            config.env.cwd = path.parent().ok_or(NotAFile)?.to_path_buf();
        }
        config.target.root = config.env.change_cwd(config.target.root.as_path());
        config.target.output = config.env.change_cwd(config.target.output.as_path());
        for req in config.req.iter_mut() {
            if let Some(ref mut path) = req.path {
                *path = config.env.change_cwd(path.as_path());
            }
        }
        Ok(config)
    }
}
