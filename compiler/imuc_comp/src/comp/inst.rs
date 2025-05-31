use crate::comp::config;
use crate::prelude::*;
use std::sync::OnceLock;

/// A compiler instance that takes in a module config and compiles it
#[derive(Default)]
pub struct CompInst {
    pub config: config::Comp,
    req: Vec<CompInst>,
    req_loaded: bool,
    ast: OnceLock<ast::module::Module>,
    ir: OnceLock<ir::module::Module>,
    ctx: OnceLock<ctx::ctx::Ctx>,
}

impl CompInst {
    /// Creates a compiler instance with the given config, and delays reading in actual content
    pub fn new(config: config::Comp) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Loads requirements from the config, if not previously done.
    pub fn load_req(&mut self) {
        if !self.req_loaded {
            info!("Loading requirements from module");
            self.req = self
                .config
                .req
                .iter()
                .filter_map(|req| {
                    let path = req
                        .path
                        .to_owned()
                        .or_else(|| crate::env::PATH_VAR.query(&req.name))
                        .filter(|path| path.exists() && path.is_file());
                    info!("Requirement name is {:?}, path is {:?}", req.name, path);
                    if let Some(path) = path {
                        let mut config = config::Comp::read_file(path.as_path())
                            .inspect_err(|err| {
                                error!(
                                    "Error when reading module config from file {:?}: {}",
                                    path, err
                                );
                            })
                            .ok()?;
                        config.target.output_dir = self.config.target.output_dir.clone();
                        Some(CompInst::new(config))
                    } else {
                        if let Some(path) = req.path.as_ref() {
                            error!("Unable to find required path {:?}", path);
                        } else {
                            error!("Unable to find required module with name {:?}", req.name);
                        }
                        None
                    }
                })
                .collect();
            self.req_loaded = true;
        }
    }

    /// Reads in file content and parse the module, if not previously done.
    /// Calling this function multiple times will only trigger parsing once
    pub fn parse(&mut self) -> &ast::module::Module {
        self.load_req();
        while let Some(mut req) = self.req.pop() {
            req.compile();
        }
        todo!("Parsing")
    }

    pub fn compile(&mut self) {}
}
