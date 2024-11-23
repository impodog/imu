use crate::prelude::*;

pub struct ElemExprRule;

impl Rule for ElemExprRule {
    type Output = expr::Expr;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        todo!()
    }
}