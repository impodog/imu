use std::path::PathBuf;

use crate::comp::config;
use crate::prelude::*;
use imuc_lexer::Filename;

/// A compiler instance that takes in a module config and compiles it
#[derive(Default)]
pub struct CompInst {
    pub config: config::Comp,

    req: Vec<CompInst>,
    req_loaded: bool,
    failed: bool,
}

impl CompInst {
    /// Creates a compiler instance with the given config, and delays reading in actual content
    pub fn new(config: config::Comp) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Returns if the compilation is failed at this point
    pub fn failed(&self) -> bool {
        self.failed
    }

    /// Loads requirements from the config, if not previously done.
    pub fn load_req(&mut self) {
        if !self.req_loaded {
            debug!(
                "Loading requirements from module {}...",
                self.config.target.module
            );
            self.req = self
                .config
                .req
                .iter()
                .filter_map(|req| {
                    let path = req
                        .path
                        .to_owned()
                        .or_else(|| crate::env::PATH_VAR.query(&req.name))
                        .map(|path| self.config.env.change_cwd(path.as_path()))
                        .filter(|path| path.exists() && path.is_file());
                    debug!("Requirement name is {:?}, path is {:?}", req.name, path);
                    if let Some(path) = path {
                        let mut config = config::Comp::read_file(path.as_path())
                            .inspect_err(|err| {
                                error!(
                                    "Error when reading module config from file {path:?}: {err:?}",
                                );
                            })
                            .ok()?;
                        if config.target.module != req.name {
                            warn!(
                                "Target name is {}, while the required name is {}",
                                config.target.module, req.name
                            );
                        }
                        config.target.output = self.config.target.output.clone();
                        Some(CompInst::new(config))
                    } else {
                        if let Some(path) = req.path.as_ref() {
                            error!("Unable to find required path {path:?}");
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

    /// Creates self.config.target.output directory, if not already
    fn create_output(&self) -> Result<()> {
        if !self.config.target.output.exists() {
            std::fs::create_dir(self.config.target.output.as_path())?;
            Ok(())
        } else if self.config.target.output.is_file() {
            Err(NotADir(self.config.target.output.clone()).into())
        } else {
            Ok(())
        }
    }

    /// Reads in file content and parse the module, returning the parse result
    pub fn parse(&mut self) -> Option<ast::module::Module> {
        self.load_req();
        while let Some(mut req) = self.req.pop() {
            req.compile();
            if req.failed() {
                self.failed = true;
                return None;
            }
        }

        let filename = Filename::new(self.config.target.root.as_os_str().to_string_lossy());
        let work = || -> Result<ast::module::Module> {
            use imuc_parser::Rule;

            let content = crate::file::FILE_MAP
                .query(filename)
                .map_err(|err| err.context(filename.get()))?;

            let reader = imuc_lexer::Reader::new(content.content().chars());
            let file_reader = imuc_parser::FileReader::new(filename, content.content(), reader);

            log::debug!("Contents of {:?} loaded, now parsing..", filename.get());

            let mut parser = imuc_parser::Parser::new(file_reader);
            parser.resolver.insert(&self.config.target.output)?;
            let module = imuc_rules::rules::ModuleRules::default()
                .parse(&mut parser)?
                .expect("ModuleRules should not return None");
            Ok(module)
        };
        work()
            .inspect_err(|err| {
                self.failed = true;
                error!("In parsing details of {:?}, {:?}", filename.get(), err)
            })
            .ok()
    }

    /// Compiles the whole module and output its content to the specified target dir
    /// The compiler will quit if errors are encountered
    pub fn compile(&mut self) -> Option<()> {
        use imuc_gen::Convert;
        use imuc_ir::io::Rw;

        info!("{}: Initializing compiler", self.config.target.module);

        if let Err(err) = self.create_output() {
            error!(
                "Unable to create output path {:?}: {:?}",
                self.config.target.output, err
            );
        }

        let ast = self.parse()?;

        info!(
            "{}: AST is generated, now doing middle-end",
            self.config.target.module
        );

        let mut ctx = ctx::ctx::Ctx::new(self.config.target.module.clone());
        imuc_gen::convs::SubmoduleConv.convert(&mut ctx, &ast).ok();
        ctx.make_entry_fun();

        let any_error = {
            let mut lock = ctx.error_queue.write().unwrap();
            let mut any_error = false;
            for err in std::mem::take(&mut *lock).into_iter() {
                if err.severity >= errors::ctx::Severity::Error {
                    any_error = true;
                }
                crate::file::FILE_MAP.report_error(err);
            }
            any_error
        };
        if any_error {
            self.failed = true;
            return None;
        }

        info!(
            "{}: Middle-end done without errors, now generating module",
            self.config.target.module
        );

        let module = ir::module::Module {
            ty: ctx.ty.extract_map(),
            fun: std::mem::take(&mut ctx.fun).into_map(),
        };

        debug!(
            "{}: Compilation done. Now exporting...",
            self.config.target.module
        );
        let output_dir = self
            .config
            .target
            .output
            .join(self.config.target.module.as_str());
        if output_dir.exists() {
            if !output_dir.is_dir() {
                error!("Output dir {output_dir:?} is created and not a dir");
                return None;
            }
        } else if let Err(err) = std::fs::create_dir(&output_dir) {
            error!("Unable to create output dir: {err}");
            return None;
        }
        // Output header & functions separately
        let (header, funs) = module.split();
        let header_path = output_dir.join("lib.iuh");
        let source_path = output_dir.join("lib.iuc");

        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&header_path)
        {
            Ok(header_file) => {
                if let Err(err) = header.write(header_file) {
                    error!(
                        "When writing header of {}, {:?}",
                        self.config.target.module, err
                    );
                    self.failed = true;
                    return None;
                }
            }
            Err(err) => {
                error!("When opening header file {header_path:?}, {err:?}");
                self.failed = true;
                return None;
            }
        }

        match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&source_path)
        {
            Ok(mut source_file) => {
                for fun in funs.into_iter() {
                    if let Err(err) = fun.write(&mut source_file) {
                        error!(
                            "When writing source of {}, {:?}",
                            self.config.target.module, err
                        );
                        self.failed = true;
                        return None;
                    }
                }
            }
            Err(err) => {
                error!("When opening source file {source_path:?}, {err:?}");
                self.failed = true;
                return None;
            }
        }

        Some(())
    }
}

#[derive(Debug)]
struct NotADir(PathBuf);
impl std::fmt::Display for NotADir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Output path {:?} is not a dir", self.0)
    }
}
impl std::error::Error for NotADir {}
