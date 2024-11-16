use crate::expr::Body;
use crate::module::Public;
use crate::pat::Pat;
use crate::prim::Prim;

/// An item definition directly used in modules, containing different kinds
pub struct Item {
    pub public: Public,
    pub templ: Vec<Templ>,
    pub name: crate::StrRef,
    pub kind: ItemKind,
}

pub enum Templ {
    Item(TemplItem),
    Unused,
}

pub struct TemplItem {
    pub name: crate::StrRef,
    pub req: Vec<crate::pat::Type>,
}

/// The internal data of an [`Item`], representing functions, customs types or constant values
pub enum ItemKind {
    Fun(Fun),
    Cus(Cus),
    For(For),
    Val(Val),
}

/// A function definition with arguments and body
pub struct Fun {
    pub args: Pat,
    pub body: Body,
}

/// A custom compound type definition
pub struct Cus {
    pub elem: Pat,
}

/// A list of implementations for a type
pub struct For {
    pub items: Vec<Item>,
}

/// A constant value of primitive
pub struct Val {
    pub val: Prim,
}
