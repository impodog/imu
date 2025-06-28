use crate::prelude::*;
use imuc_lexer::token::{Ident, Symbol};

/// Outputs the prefix to type/value names. This rule is lazy, it only consumes the prefix from the
/// parser, store it in the queue and return whether it is successful by [`Option`]. If the rule is
/// called multiple times without popping the prefix with [`Parser::pop_prefix`], no action will be
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
        if parser
            .peek()?
            .is_some_and(|input| matches!(input.kind, TokenKind::Prefix))
        {
            return Ok(Some(()));
        }

        let mut prefix: Vec<StrRef> = Vec::new();
        while let Some(next) = parser.peek_nth(1)? {
            if next.kind == TokenKind::Symbol(Symbol::DblColon) {
                let name = parser.next_expected(&TokenKind::Ident(Ident::Value))?;
                // Remove double colon
                parser
                    .next_token()?
                    .expect("Should contain a token after peeking");
                prefix.push(parser.look_up.insert(name.value));
            } else {
                break;
            }
        }
        if prefix.is_empty() {
            Ok(None)
        } else {
            parser.push_prefix(name::Prefix::new(
                nonempty::NonEmpty::from_vec(prefix)
                    .expect("Prefix should not be empty after checking"),
            ));
            Ok(Some(()))
        }
    }
}
