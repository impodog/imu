use crate::prelude::*;
use imuc_ast::{
    module::{Import, ImportDir, ImportItemKind},
    name::{Prefix, PrefixFirst},
};
use imuc_lexer::token::{BinOp, Ident, Keyword, Pair, Symbol};

// Used by single imports either in a list or appear individually, allowing wildcard omitting prefix(::*) or wildcard with prefix(::+)
lazy_tokens!(
    ImportTokens,
    Ident::Value,
    Ident::Type,
    BinOp::Mul,
    BinOp::Add
);

struct ImportItemRule {
    prefix: Prefix,
}

impl ImportItemRule {
    fn into_item(token: imuc_parser::ParserInput<'_>, str: StrRef) -> module::ImportItemKind {
        let f = match token.kind {
            TokenKind::Ident(ident) => match ident {
                Ident::Type => module::ImportItemKind::Type,
                Ident::Value => module::ImportItemKind::Value,
                _ => filtered!(),
            },
            TokenKind::BinOp(BinOp::Add) => return module::ImportItemKind::WildcardWithPrefix,
            TokenKind::BinOp(BinOp::Mul) => return module::ImportItemKind::Wildcard,
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

fn parse_import_item<'s, I>(
    parser: &mut Parser<'s, I>,
    prefix: Prefix,
) -> Result<module::ImportItem>
where
    I: ParserSequence<'s>,
{
    let cursor_begin = parser.relative_cursor();
    let item = parser.next_expected(&ImportTokens)?;
    let kind = ImportItemRule::into_item(item, parser.look_up.insert(item.value));

    let alias = if matches!(
        kind,
        ImportItemKind::Wildcard | ImportItemKind::WildcardWithPrefix
    ) {
        // If the prefix has more than just a head
        if !prefix.is_empty() {
            return Err(parser.map_err(errors::SyntaxError::ExpectedBefore {
                expect: "just one name".to_owned(),
                before: item.kind,
            }));
        }
        None
    } else {
        ImportItemRule::next_alias(item.kind, parser)?
    };

    Ok(module::ImportItem {
        prefix,
        kind,
        alias,
        span: parser.file_info().into_span(cursor_begin),
    })
}

impl Rule for ImportItemRule {
    type Output = Vec<module::ImportItem>;
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let ImportItemRule { prefix } = self;
        if parser.next_if(&TokenKind::Pair(Pair::LeftParen))?.is_some() {
            let mut list = Self::Output::new();
            let mut comma = true;
            loop {
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightParen))?
                    .is_some()
                {
                    break;
                } else if !comma {
                    return Err(parser.map_err(errors::SyntaxError::ExpectedToken {
                        expect: TokenKind::Pair(Pair::RightParen),
                    }));
                }

                list.push(parse_import_item(parser, prefix.clone())?);

                comma = parser.next_if(&TokenKind::Symbol(Symbol::Comma))?.is_some();
            }
            Ok(Some(list))
        } else {
            Ok(Some(vec![parse_import_item(parser, prefix)?]))
        }
    }
}

/// Imports a single "use" statement from the parser
pub struct ImportRule;

impl Rule for ImportRule {
    type Output = Import;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        let cursor_begin = parser.relative_cursor();

        rules::PrefixRule.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedAfter {
                expect: "module name and item".to_owned(),
                after: TokenKind::Keyword(Keyword::Use),
            })
        })?;
        let prefix = parser
            .pop_prefix()
            .expect("Should contain a prefix after calling PrefixRule");
        let dir = match prefix.first {
            PrefixFirst::Name(ref path) => {
                let module = parser
                    .resolver
                    .query(std::borrow::Cow::Borrowed(path.as_str()))?;
                ImportDir::External(module.base().to_path_buf())
            }
            PrefixFirst::Loc => ImportDir::Loc,
        };

        let item = ImportItemRule { prefix }.parse(parser)?.ok_or_else(|| {
            parser.map_err(errors::SyntaxError::ExpectedIn {
                expect: "Item".to_owned(),
                context: "import statement".to_owned(),
            })
        })?;
        let _ = parser.next_expected(&TokenKind::Semicolon);
        Ok(Some(module::Import {
            dir,
            item,
            span: parser.file_info().into_span(cursor_begin),
        }))
    }
}
