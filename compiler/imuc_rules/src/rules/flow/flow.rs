use crate::prelude::*;

pub struct FlowRule;

impl Rule for FlowRule {
    type Output = flow::Flow;

    fn parse<'s, I>(self, parser: &mut Parser<'s, I>) -> Result<Option<Self::Output>>
    where
        I: ParserSequence<'s>,
    {
        if let Some(flow) = rules::IfRule.parse(parser)? {
            Ok(Some(flow::Flow::If(flow)))
        } else if let Some(flow) = rules::LoopRule.parse(parser)? {
            Ok(Some(flow::Flow::Loop(flow)))
        } else {
            Ok(None)
        }
    }
}
