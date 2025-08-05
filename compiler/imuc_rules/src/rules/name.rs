use crate::prelude::*;
use imuc_lexer::token::{Ident, Keyword, Symbol};

/// Outputs the prefix to type/value names. This rule is lazy, it only consumes the prefix from the
/// parser, store it in the queue and return whether it is successful by `Option`. If the rule is
/// called multiple times without popping the prefix with `Parser::pop_prefix`, no action will be
/// done.
///
/// This rule does not output anything, because it inserts the prefix into the parser queue
pub struct PrefixRule;

impl Rule for PrefixRule {
    type Output = ();
    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        // Test if a prefix already exists
        if parser.has_prefix() {
            return Ok(Some(()));
        }

        let mut prefix: Option<name::Prefix> = None;
        while let Some(next) = parser.peek_nth(1)? {
            if next.kind == TokenKind::Symbol(Symbol::DblColon) {
                if let Some(name) = parser.next_if(&TokenKind::Ident(Ident::Value))? {
                    let value = parser.look_up.insert(name.value);
                    if let Some(ref mut prefix) = prefix {
                        prefix.push(value);
                    } else {
                        prefix = Some(name::Prefix::new(name::PrefixFirst::Name(value)));
                    }
                } else if parser.next_if(&TokenKind::Keyword(Keyword::Loc))?.is_some() {
                    if prefix.is_some() {
                        return Err(errors::SyntaxError::ExpectedAfter {
                            expect: "module name".to_owned(),
                            after: TokenKind::Symbol(Symbol::DblColon),
                        }
                        .into());
                    } else {
                        prefix = Some(name::Prefix::new(name::PrefixFirst::Loc));
                    }
                } else {
                    return Err(errors::SyntaxError::ExpectedBefore {
                        expect: "module name or `loc`".to_owned(),
                        before: TokenKind::Symbol(Symbol::DblColon),
                    }
                    .into());
                }
                // Remove double colon
                parser
                    .next_token()?
                    .expect("Should contain a token after peeking");
            } else {
                break;
            }
        }
        if let Some(prefix) = prefix {
            parser.push_prefix(prefix);
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
