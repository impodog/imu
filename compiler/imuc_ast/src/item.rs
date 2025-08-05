use crate::expr::Body;
use crate::module::{Import, Public};
use crate::pat::{Pat, Type};
use crate::prim::Prim;

/// An item definition directly used in modules, containing different kinds
pub struct Item {
    pub public: Public,
    pub name: imuc_lexer::StrRef,
    pub kind: ItemKind,
}

pub enum Templ {
    Item(TemplItem),
    Unused,
}

pub struct TemplItem {
    pub name: imuc_lexer::StrRef,
    pub req: Vec<crate::pat::Type>,
}

/// The internal data of an `Item`, representing functions, customs types or constant values
pub enum ItemKind {
    Fun(Fun),
    Cus(Cus),
    For(For),
    Val(Val),
    Use(Import),
}

/// A function definition with arguments and body
pub struct Fun {
    pub param: Pat,
    pub ret: Option<Type>,
    pub body: Body,
}

impl Fun {
    pub fn span(&self) -> imuc_lexer::Span {
        imuc_lexer::Span {
            file: self.param.span.file,
            start: self.param.span.start,
            end: self.body.span.end,
        }
    }
}

/// A custom compound type definition
pub struct Cus {
    pub elem: Pat,
}

/// A list of implementations for a type
pub struct For {
    pub ty: Type,
    pub items: Vec<Item>,
}

/// A constant value of primitive
pub struct Val {
    pub val: Prim,
}
