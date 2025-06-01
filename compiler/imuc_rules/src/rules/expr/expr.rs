use crate::prelude::*;
use crate::Priority;
use imuc_lexer::token::Pair;
use imuc_parser::TokenKindSet;

lazy_tokens!(EndTokens, Pair::RightParen, Pair::RightBracket, Pair::RightBrace and Semicolon);

/// [`Self::end`] defines the token to end the expression when meet
///
/// The expression will end anyway if it meets open right brackets ')', ']', '}' or ';' (as a suffix and will not be consumed)
pub struct ExprRule<T>
where
    T: for<'a> TokenKindSet<'a>,
{
    pub end: T,
}

/// A local struct that holds both the expression kind and its beginning cursor
struct ExprItem {
    expr: expr::Expr,
    cursor: Cursor,
}

/// A local struct that holds both the operator and its beginning cursor
#[derive(Debug)]
struct OpItem {
    op: TokenKind,
    cursor: Cursor,
}

fn merge_symbols<'s, I>(
    parser: &mut Parser<'s, I>,
    op_item: OpItem,
    stack: &mut Vec<ExprItem>,
) -> Result<()>
where
    I: ParserSequence<'s>,
{
    match op_item.op {
        TokenKind::UnOp(op) => {
            let ExprItem {
                expr,
                cursor: _cursor,
            } = stack.pop().ok_or(errors::SyntaxError::TooManyOp)?;
            let expr = expr::Expr::UnExpr(expr::UnExpr {
                op,
                val: Box::new(expr),
                span: parser.file_info().into_span(op_item.cursor),
            });
            stack.push(ExprItem {
                expr,
                cursor: op_item.cursor,
            });
        }
        TokenKind::BinOp(op) => {
            let ExprItem {
                expr: lhs_expr,
                cursor: lhs_cursor,
            } = stack.pop().ok_or(errors::SyntaxError::TooManyOp)?;
            let ExprItem {
                expr: rhs_expr,
                cursor: _rhs_cursor,
            } = stack.pop().ok_or(errors::SyntaxError::TooManyOp)?;
            let expr = expr::Expr::BinExpr(expr::BinExpr {
                op,
                lhs: Box::new(lhs_expr),
                rhs: Box::new(rhs_expr),
                span: parser.file_info().into_span(lhs_cursor),
            });
            stack.push(ExprItem {
                expr,
                cursor: lhs_cursor,
            });
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
        let mut stack: Vec<ExprItem> = Vec::new();
        let mut op: Vec<OpItem> = Vec::new();
        // Determines whether to parse the expression as function call
        let mut prev_is_expr = false;
        loop {
            let cursor_begin = parser.relative_cursor();

            // FIXME: Special test used by if ... `{`, should we make it more uniform?
            if prev_is_expr
                && parser
                    .peek()?
                    .is_some_and(|input| end.contains(&input.kind))
            {
                break;
            }

            if let Some(expr) = rules::ElemExprRule.parse(parser)? {
                if prev_is_expr {
                    let ExprItem {
                        expr: prev_expr,
                        cursor: prev_cursor,
                    } = stack
                        .pop()
                        .expect("when prev_is_expr, stack should not be empty");
                    let call = ExprItem {
                        expr: expr::Expr::Call(expr::Call {
                            func: Box::new(prev_expr),
                            args: Box::new(expr),
                            span: parser.file_info().into_span(prev_cursor),
                        }),
                        cursor: prev_cursor,
                    };
                    stack.push(call);
                    // No need to update prev_is_expr since it is already true
                    // This also allows chained function calls
                } else {
                    let expr = ExprItem {
                        expr,
                        cursor: cursor_begin,
                    };
                    stack.push(expr);
                    prev_is_expr = true;
                }
            } else {
                prev_is_expr = false;

                let input = parser.peek()?;
                if let Some(input) = input {
                    match input.kind {
                        TokenKind::UnOp(_) | TokenKind::BinOp(_) => {
                            while op.last().is_some_and(|op| {
                                if op.op.is_right() {
                                    op.op.priority() < input.kind.priority()
                                } else {
                                    op.op.priority() <= input.kind.priority()
                                }
                            }) {
                                let op = op.pop().expect("op should not be empty after checking");
                                merge_symbols(parser, op, &mut stack)
                                    .map_err(|err| parser.map_err(err))?;
                            }
                            op.push(OpItem {
                                op: input.kind,
                                cursor: cursor_begin,
                            });
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
                    let _ = parser
                        .next_some()
                        .expect("parser should not be EOF after peeking a symbol");
                } else {
                    break;
                }
            }
        }
        for op in op.into_iter().rev() {
            merge_symbols(parser, op, &mut stack).map_err(|err| parser.map_err(err))?;
        }
        match stack.len() {
            0 => unreachable!("stack should be empty at this point"),
            1 => Ok(Some(stack.into_iter().next().unwrap().expr)),
            _ => parser.error(errors::SyntaxError::TooFewOp),
        }
    }
}
