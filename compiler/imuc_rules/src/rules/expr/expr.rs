use crate::prelude::*;

pub struct ExprRule;

impl Rule for ExprRule {
    type Output = expr::Expr;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!();
    }
}
