use crate::mem::*;
use crate::prelude::*;
use std::ops::Deref;

pub struct CmdBody {
    list: Vec<Cmd>,
}
impl Deref for CmdBody {
    type Target = [Cmd];
    fn deref(&self) -> &Self::Target {
        &self.list
    }
}

impl CmdBody {
    pub fn new(list: Vec<Cmd>) -> Self {
        Self { list }
    }
}

pub enum Cmd {
    Alloc(Bytes),
}
