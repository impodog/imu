use crate::comp::config;
use crate::prelude::*;
use imuc_lexer::Filename;

/// A compiler instance that takes in a module config and compiles it
#[derive(Default)]
pub struct CompInst {
    pub config: config::Comp,
    filename: Filename,

    req: Vec<CompInst>,
    req_loaded: bool,
    failed: bool,
}

impl CompInst {
    /// Creates a compiler instance with the given config, and delays reading in actual content
    pub fn new(config: config::Comp, filename: Filename) -> Self {
        Self {
            config,
            filename,
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
                        if config.target.module != req.name {
                            warn!(
                                "Target name is {}, while the required name is {}",
                                config.target.module, req.name
                            );
                        }
                        config.target.output = self.config.target.output.clone();
                        Some(CompInst::new(
                            config,
                            Filename::new(
                                path.into_os_string()
                                    .into_string()
                                    .expect("Expected valid utf-8 path"),
                            ),
                        ))
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

        let work = || -> Result<ast::module::Module> {
            use imuc_parser::Rule;

            let content = crate::file::FILE_MAP.query(self.filename)?;
            let reader = imuc_lexer::Reader::new(content.content().chars());
            let file_reader =
                imuc_parser::FileReader::new(self.filename, content.content(), reader);
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
                error!("When parsing {:?}, {}", self.filename.get(), err)
            })
            .ok()
    }

    /// Compiles the whole module and output its content to the specified target dir
    /// The compiler will quit if errors are encountered
    pub fn compile(&mut self) -> Option<()> {
        use imuc_gen::Convert;
        use imuc_ir::io::Rw;

        debug!("Compiling {}", self.config.target.module);

        let ast = self.parse()?;
        let mut ctx = ctx::ctx::Ctx::new(self.config.target.module.clone());
        imuc_gen::convs::SubmoduleConv.convert(&mut ctx, &ast).ok();
        for err in std::mem::take(&mut *ctx.error_queue.write().unwrap()).into_iter() {
            crate::file::FILE_MAP.report_error(err);
        }
        let module = ir::module::Module {
            ty: ctx.ty.to_map(),
            fun: std::mem::take(&mut ctx.fun).into_map(),
        };
        let (header, funs) = module.split();
        let header_path = self
            .config
            .target
            .output
            .join(self.config.target.root.with_extension("iuh"));
        let source_path = self
            .config
            .target
            .output
            .join(self.config.target.root.with_extension("iuc"));

        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&header_path)
        {
            Ok(header_file) => {
                if let Err(err) = header.write(header_file) {
                    error!(
                        "When writing header of {}, {}",
                        self.config.target.module, err
                    );
                    self.failed = true;
                    return None;
                }
            }
            Err(err) => {
                error!("When opening header file {:?}, {}", header_path, err);
                self.failed = true;
                return None;
            }
        }

        match std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&source_path)
        {
            Ok(mut source_file) => {
                for fun in funs.into_iter() {
                    if let Err(err) = fun.write(&mut source_file) {
                        error!(
                            "When writing source of {}, {}",
                            self.config.target.module, err
                        );
                        self.failed = true;
                        return None;
                    }
                }
            }
            Err(err) => {
                error!("When opening source file {:?}, {}", source_path, err);
                self.failed = true;
                return None;
            }
        }

        Some(())
    }
}
