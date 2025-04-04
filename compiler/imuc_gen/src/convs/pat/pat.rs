use crate::prelude::*;
use ast::pat::*;

// FIXME: Is this conversion really needed?
pub struct PatConv;

impl Converter for PatConv {
    type Input = Pat;
}
