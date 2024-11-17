use crate::prelude::*;
use imuc_lexer::token::{Ident, Keyword, Pair, Symbol};
use std::path::PathBuf;

lazy_tokens!(ImportTokens, Ident::Value, Ident::Type);

pub struct ImportItemRule;

impl ImportItemRule {
    fn into_item(token: imuc_parser::ParserInput<'_>, str: StrRef) -> module::ImportItemKind {
        let f = match token.kind {
            TokenKind::Ident(ident) => match ident {
                Ident::Type => module::ImportItemKind::Type,
                Ident::Value => module::ImportItemKind::Value,
                _ => filtered!(),
            },
            _ => filtered!(),
        };
        f(str)
    }

    fn next_alias<'s, I>(item_kind: TokenKind, parser: &mut Parser<'s, I>) -> Result<Option<StrRef>>
    where
        I: ParserSequence<'s>,
    {
        let alias = if parser.next_if(&TokenKind::Keyword(Keyword::As))?.is_some() {
            let alias = parser.next_expected(&ImportTokens)?;
            if alias.kind != item_kind {
                return Err(parser.map_err(errors::SyntaxError::AliasMismatch {
                    item: item_kind,
                    alias: alias.kind,
                }));
            } else {
                Some(parser.look_up.insert(alias.value))
            }
        } else {
            None
        };
        Ok(alias)
    }
}

impl Rule for ImportItemRule {
    type Output = Vec<module::ImportItem>;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftBrace))?.is_some() {
            let mut list = Self::Output::new();
            let mut comma = true;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightBrace))?
                    .is_some()
                {
                    break;
                } else if !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightBrace),
                    }));
                }

                let item = parser.next_expected(&ImportTokens)?;
                let kind = Self::into_item(item, parser.look_up.insert(item.value));

                let alias = Self::next_alias(item.kind, parser)?;

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();

                list.push(module::ImportItem { kind, alias });
            }
            Ok(Some(list))
        } else if let Some(item) = parser.next_if(&ImportTokens)? {
            let alias = Self::next_alias(item.kind, parser)?;
            Ok(Some(vec![module::ImportItem {
                kind: Self::into_item(item, parser.look_up.insert(item.value)),
                alias,
            }]))
        } else {
            Ok(None)
        }
    }
}

/// Imports a single "use" statement from the parser
pub struct ImportRule<'a> {
    pub import: &'a mut Vec<module::Import>,
}

impl<'a> Rule for ImportRule<'a> {
    type Output = ();

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Keyword(Keyword::Use))?.is_some() {
            let module = parser.next_expected(&TokenKind::Ident(Ident::Value))?;
            let module = imuc_path::Module::new(PathBuf::from(module.value)).resolve();

            let mut module_ref = &module;
            let file = loop {
                let _ = parser.next_expected(&TokenKind::Symbol(Symbol::Dot))?;
                let next = parser.next_if(&TokenKind::Ident(Ident::Value))?;
                if let Some(next) = next {
                    let sub = module_ref.get(next.value).ok_or_else(|| {
                        parser.map_err(errors::PathError::ModuleNotFound(next.value.to_owned()))
                    })?;
                    match sub {
                        imuc_path::SubModule::File(file) => {
                            break Some(file);
                        }
                        imuc_path::SubModule::Module(module) => {
                            module_ref = module;
                        }
                    }
                } else {
                    break None;
                }
            };
            match file {
                Some(file) => {
                    let _ = parser.next_expected(&TokenKind::Symbol(Symbol::Dot))?;
                    let item = ImportItemRule.parse(parser)?.ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Item".to_owned(),
                            context: "import statement".to_owned(),
                        })
                    })?;
                    self.import.push(module::Import {
                        file: file.clone(),
                        item,
                    });
                    let _ = parser.next_expected(&TokenKind::Semicolon);
                }
                None => {
                    // TODO: Add reference alias
                    return Err(parser.map_err(errors::PathError::BrokenPath));
                }
            }
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
