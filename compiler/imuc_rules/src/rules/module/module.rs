use crate::prelude::*;
use imuc_ast::module::SubNode;
use imuc_lexer::token::{Ident, Keyword};
use std::path::{Path, PathBuf};

enum LocalRef<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> LocalRef<'a, T> {
    fn as_ref<'s>(&'s self) -> &'a T
    where
        's: 'a,
    {
        match self {
            Self::Borrowed(borrowed) => borrowed,
            Self::Owned(owned) => owned,
        }
    }
}

#[derive(Default)]
pub struct ModuleRules<'module> {
    local: Option<&'module imuc_path::Module>,
    is_file: bool,
}

/// Opens the file, creates a parser and parse its contents
fn parse_file(rules: ModuleRules<'_>, path: &Path) -> Result<module::Module> {
    let content = std::fs::read_to_string(path)?;
    let filename =
        imuc_lexer::Filename::new(String::from_utf8_lossy(path.as_os_str().as_encoded_bytes()));
    let reader = imuc_parser::FileReader::new(
        filename,
        content.as_str(),
        imuc_lexer::Reader::new(content.chars()),
    );
    let mut parser = imuc_parser::Parser::new(reader);
    let module = rules
        .parse(&mut parser)?
        .expect("ModuleRules should not return None");
    Ok(module)
}

impl Rule for ModuleRules<'_> {
    type Output = module::Module;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let ModuleRules { local, is_file } = self;
        let sub_nodes = if is_file {
            Vec::default()
        } else {
            let path = PathBuf::from(parser.file_info().file.get().as_str());
            let dir = path
                .parent()
                .expect("input file should have a parent")
                .to_path_buf();
            let local = if let Some(local) = local {
                LocalRef::Borrowed(local)
            } else {
                let local = imuc_path::Module::new(dir.clone()).resolve();
                LocalRef::Owned(local)
            };

            let mut sub_nodes = Vec::new();
            while parser.next_if(&TokenKind::Keyword(Keyword::Mod))?.is_some() {
                let cursor_begin = parser.relative_cursor();

                let name = parser.next_expected(&TokenKind::Ident(Ident::Value))?;
                match local.as_ref().get(name.value) {
                    Some(sub_module) => match sub_module {
                        imuc_path::SubModule::File(file) => {
                            let module = parse_file(
                                ModuleRules {
                                    local: None,
                                    is_file: true,
                                },
                                file.path(),
                            )
                            .map_err(|err| parser.map_err(err))?;
                            sub_nodes.push(SubNode {
                                name: name.value.into(),
                                module,
                                span: parser.file_info().into_span(cursor_begin),
                            });
                        }
                        imuc_path::SubModule::Module(module) => {
                            let sub_dir = dir.join(name.value);
                            let sub_file_path = sub_dir.join("mod.rs");
                            if sub_file_path.exists() {
                                let module = parse_file(
                                    ModuleRules {
                                        local: Some(module),
                                        is_file: false,
                                    },
                                    sub_file_path.as_path(),
                                )
                                .map_err(|err| parser.map_err(err))?;
                                sub_nodes.push(SubNode {
                                    name: name.value.into(),
                                    module,
                                    span: parser.file_info().into_span(cursor_begin),
                                });
                            } else {
                                return Err(errors::PathError::ModuleNotFound(
                                    name.value.to_owned(),
                                )
                                .into());
                            }
                        }
                    },
                    None => {
                        return Err(parser
                            .map_err(errors::PathError::ModuleNotFound(name.value.to_owned())))
                    }
                }
                while parser.next_if(&TokenKind::Semicolon)?.is_some() {}
            }
            sub_nodes
        };
        let items = {
            let mut items = Vec::new();
            while let Some(item) = rules::ItemRule.parse(parser)? {
                items.push(item);
            }
            items
        };
        Ok(Some(module::Module { sub_nodes, items }))
    }
}
