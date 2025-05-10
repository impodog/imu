use crate::prelude::*;
use ast::flow::Flow;
use convs::expr::ExprSolver;

pub struct FlowConv {
    pub solver: ExprSolver,
}

impl Converter for FlowConv {
    type Input = Flow;
}

impl Convert<Value> for FlowConv {
    fn convert(self, ctx: &mut Ctx, input: &Self::Input) -> Result<Value> {
        match input {
            Flow::IfElse(ifelse_stmt) => {
                todo!()
            }
            Flow::Loop(loop_stmt) => {
                todo!()
            }
        }
    }
}
