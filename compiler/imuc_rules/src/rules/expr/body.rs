use crate::prelude::*;
use imuc_lexer::token::Pair;

pub struct BodyRule;

#[derive(Default)]
struct BodyElem {
    bind: Vec<bind::Bind>,
    body: Vec<expr::Expr>,
    unit: bool,
}

impl From<BodyElem> for expr::Expr {
    fn from(value: BodyElem) -> Self {
        Self::Body(expr::Body {
            bind: value.bind,
            body: value.body,
            unit: value.unit,
        })
    }
}

impl Rule for BodyRule {
    type Output = expr::Body;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if parser.next_if(&TokenKind::Pair(Pair::LeftBrace))?.is_some() {
            let mut stack = vec![BodyElem::default()];
            let mut bind_seq = true;
            let mut unit = false;
            loop {
                while parser.next_if(&TokenKind::Semicolon)?.is_some() {
                    unit = true;
                }
                if parser
                    .next_if(&TokenKind::Pair(Pair::RightBrace))?
                    .is_some()
                {
                    break;
                }
                unit = false;
                if let Some(bind) = rules::BindRule.parse(parser)? {
                    if bind_seq {
                        stack.last_mut().unwrap().bind.push(bind);
                    } else {
                        stack.push(BodyElem {
                            bind: vec![bind],
                            body: Default::default(),
                            unit: false,
                        });
                        bind_seq = true;
                    }
                } else {
                    let expr = rules::ExprRule { end: () }.parse(parser)?.ok_or_else(|| {
                        parser.map_err(errors::SyntaxError::ExpectedIn {
                            expect: "Expr or Bind".to_owned(),
                            context: "expression body".to_owned(),
                        })
                    })?;
                    stack.last_mut().unwrap().body.push(expr);
                    bind_seq = false;
                }
            }
            // Set the unit-ness for the most inner body
            stack.last_mut().unwrap().unit = unit;

            let body = stack.into_iter().fold(None, |inner, mut elem| {
                if let Some(inner) = inner {
                    elem.body.push(inner);
                }
                Some(elem.into())
            });
            if let expr::Expr::Body(body) = body.expect("the stack should not be empty") {
                Ok(Some(body))
            } else {
                unreachable!("the output of folding should be a body")
            }
        } else {
            Ok(None)
        }
    }
}
