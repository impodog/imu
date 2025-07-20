use crate::prelude::*;
use crate::Priority;
use imuc_lexer::token::BinOp;
use imuc_lexer::token::Pair;
use imuc_lexer::token::UnOp;
use imuc_parser::ParserInput;
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
            // NOTE: RHS comes first on top of the stack
            let ExprItem {
                expr: rhs_expr,
                cursor: _rhs_cursor,
            } = stack.pop().ok_or(errors::SyntaxError::TooManyOp)?;
            let ExprItem {
                expr: lhs_expr,
                cursor: lhs_cursor,
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

fn push_symbol<'s, I>(
    parser: &mut Parser<'s, I>,
    op: &mut Vec<OpItem>,
    stack: &mut Vec<ExprItem>,
    cursor: Cursor,
    input: ParserInput<'s>,
) -> Result<()>
where
    I: ParserSequence<'s>,
{
    while op.last().is_some_and(|op| {
        if op.op.is_right() {
            op.op.priority() < input.kind.priority()
        } else {
            op.op.priority() <= input.kind.priority()
        }
    }) {
        let op = op.pop().expect("op should not be empty after checking");
        merge_symbols(parser, op, stack).map_err(|err| parser.map_err(err))?;
    }
    op.push(OpItem {
        op: input.kind,
        cursor,
    });
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
        // Used by call to check whether the expression directly follows another
        let mut prev_is_expr = false;
        loop {
            let cursor_begin = parser.relative_cursor();

            // FIXME: Special test used by if ... `{`, should we make it more uniform?
            if !stack.is_empty()
                && parser
                    .peek()?
                    .is_some_and(|input| end.contains(&input.kind))
            {
                break;
            }

            // NOTE: This prevents the two symbols @ and $ being recognized as types

            let is_elem = if !parser
                .peek()?
                .is_some_and(|input| matches!(input.kind, TokenKind::UnOp(_)))
            {
                match rules::ElemExprRule.parse(parser)? {
                    Some(expr) => {
                        let expr = ExprItem {
                            expr,
                            cursor: cursor_begin,
                        };
                        if prev_is_expr {
                            push_symbol(
                                parser,
                                &mut op,
                                &mut stack,
                                cursor_begin,
                                ParserInput {
                                    kind: TokenKind::BinOp(BinOp::Call),
                                    value: "#CALL",
                                },
                            )?;
                        }
                        stack.push(expr);
                        prev_is_expr = true;
                        true
                    }
                    _ => false,
                }
            } else {
                false
            };

            // Uses the condition returned by the previous if,
            // if not a element, operators are parsed
            if !is_elem {
                let input = parser.peek()?;
                if let Some(input) = input {
                    let kind = if prev_is_expr {
                        input.kind
                    } else {
                        match input.kind {
                            TokenKind::BinOp(BinOp::Sub) => TokenKind::UnOp(UnOp::Neg),
                            TokenKind::BinOp(BinOp::Mul) => TokenKind::UnOp(UnOp::Deref),
                            _ => input.kind,
                        }
                    };
                    match kind {
                        TokenKind::UnOp(_) | TokenKind::BinOp(_) => {
                            push_symbol(
                                parser,
                                &mut op,
                                &mut stack,
                                cursor_begin,
                                ParserInput {
                                    value: input.value,
                                    kind,
                                },
                            )?;
                        }
                        _ => {
                            if end.contains(&kind) {
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
                    prev_is_expr = false;
                } else {
                    break;
                }
            }
        }
        for op in op.into_iter().rev() {
            merge_symbols(parser, op, &mut stack).map_err(|err| parser.map_err(err))?;
        }
        match stack.len() {
            0 => Ok(Some(expr::Expr::Prim(prim::Prim::Unit))),
            1 => Ok(Some(stack.into_iter().next().unwrap().expr)),
            _ => parser.error(errors::SyntaxError::TooFewOp),
        }
    }
}
