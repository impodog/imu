use crate::prelude::*;
use ast::pat::*;

pub struct PatConv;

impl Converter for PatConv {
    type Input = Pat;
}
