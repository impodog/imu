use crate::prelude::*;
use imuc_lexer::token::Pair;
use imuc_parser::TokenKindSet;

lazy_tokens!(EndTokens, Pair::RightParen, Pair::RightBracket, Pair::RightBrace and Semicolon);

/// [`Self::end`] defines the token to end the expression when meet
///
/// The expression will end anyway if it meets open right brackets ')', ']', '}' or ';' (as a suffix)
pub struct ExprRule<T>
where
    T: for<'a> TokenKindSet<'a>,
{
    pub end: T,
}

fn merge_symbols(op: TokenKind, stack: &mut Vec<expr::Expr>) -> Result<()> {
    match op {
        TokenKind::UnOp(op) => {
            let val = stack.pop().ok_or_else(|| errors::SyntaxError::TooManyOp)?;
            stack.push(expr::Expr::UnExpr(expr::UnExpr {
                op,
                val: Box::new(val),
            }));
        }
        TokenKind::BinOp(op) => {
            let lhs = stack.pop().ok_or_else(|| errors::SyntaxError::TooManyOp)?;
            let rhs = stack.pop().ok_or_else(|| errors::SyntaxError::TooManyOp)?;
            stack.push(expr::Expr::BinExpr(expr::BinExpr {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }));
        }
        _ => {
            unreachable!("input op should be an operator")
        }
    }
    Ok(())
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
        let end = (self.end, EndTokens);
        let mut stack = Vec::new();
        let mut op: Vec<TokenKind> = Vec::new();
        loop {
            if let Some(item) = rules::ElemExprRule.parse(parser)? {
                stack.push(item);
            } else {
                let input = parser.next_some()?;
                match input.kind {
                    TokenKind::UnOp(_) | TokenKind::BinOp(_) => {
                        while op.last().is_some_and(|op| {
                            if op.is_right() {
                                op.priority() < input.kind.priority()
                            } else {
                                op.priority() <= input.kind.priority()
                            }
                        }) {
                            let op = op.pop().expect("op should not be empty after checking");
                            merge_symbols(op, &mut stack).map_err(|err| parser.map_err(err))?;
                        }
                        op.push(input.kind);
                    }
                    _ => {
                        if end.contains(&input.kind) {
                            break;
                        } else {
                            return parser.error(errors::SyntaxError::ExpectedIn {
                                expect: "Expr or Op".to_owned(),
                                context: "expression".to_owned(),
                            });
                        }
                    }
                }
            }
        }
        for op in op.into_iter().rev() {
            merge_symbols(op, &mut stack).map_err(|err| parser.map_err(err))?;
        }
        match stack.len() {
            0 => unreachable!("stack should be empty at this point"),
            1 => Ok(Some(stack.into_iter().next().unwrap())),
            _ => parser.error(errors::SyntaxError::TooFewOp),
        }
    }
}
