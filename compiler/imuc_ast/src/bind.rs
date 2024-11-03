use crate::expr::Expr;
use crate::item::Item;
use crate::pat::Pat;

/// A binding that creates links from names to code objects
pub enum Bind {
    Item(Item),
    Let(Let),
}

/// "let" binding creates a link from names to values
pub struct Let {
    pub pat: Pat,
    pub val: Expr,
}
