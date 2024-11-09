use crate::expr::Body;
use crate::module::Public;
use crate::pat::Pat;
use crate::prim::Prim;

/// An item definition directly used in modules, containing different kinds
pub struct Item {
    pub public: Public,
    pub name: String,
    pub template: Vec<String>,
    pub kind: ItemKind,
}

/// The internal data of an [`Item`], representing functions, customs types or constant values
pub enum ItemKind {
    Fun(Fun),
    Cus(Cus),
    Const(Const),
}

/// A function definition with arguments and body
pub struct Fun {
    pub args: Pat,
    pub body: Body,
}

pub struct Cus {
    pub elem: Pat,
}

pub struct Const {
    pub val: Prim,
}
