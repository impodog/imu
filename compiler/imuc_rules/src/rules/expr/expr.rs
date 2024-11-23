use crate::prelude::*;
use imuc_parser::TokenKindSet;

/// [`Self::end`] defines the token to end the expression when meet
///
/// The expression will end anyway if it meets open right brackets ')', ']', '}' or ';' (as a suffix)
pub struct ExprRule<T>
where
    T: for<'a> TokenKindSet<'a>,
{
    pub end: T,
}

impl<T> Rule for ExprRule<T>
where
    T: for<'a> TokenKindSet<'a>,
{
    type Output = expr::Expr;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!();
    }
}
