use crate::prelude::*;
use imuc_ir::cmd::{GlobalPtr, Ptr};
use imuc_ir::sym::Ty;
use nonempty::NonEmpty;

/// All context info used when converting AST to IR
///
/// An additional type map is present, containing all types with mangled names.
/// Searching directly is impossible,
pub struct Ctx {
    pub ty: super::Types,
    body: NonEmpty<super::Body>,
}

impl Ctx {
    /// Creates a new empty context of IR conversion
    pub fn new(base_name: String) -> Self {
        Self {
            ty: Default::default(),
            body: NonEmpty::new(super::Body::new(base_name, None)),
        }
    }

    /// Gets the reference to the current function body
    pub fn body_mut(&mut self) -> &mut super::Body {
        self.body.last_mut()
    }

    /// Pushes a new body of function when entering inner functions
    pub fn push_body(&mut self, name: &str, self_ty: Option<Ty>) -> &mut super::Body {
        let base_name = self.body.last().name();
        let name = format!("{base_name}.{name}");

        self.body.push(super::Body::new(name, self_ty));
        self.body.last_mut()
    }

    /// Gets the last body of functions being pushed
    pub fn pop_body(&mut self) -> Option<super::Body> {
        self.body.pop()
    }

    /// Gets reference to the nearest type of given name
    pub fn get_type(&self, name: &str) -> Option<&sym::Ty> {
        let body = self.body.last();
        for locals in body.locals_iter_rev() {
            if let Some(ty) = locals.ty.get(name) {
                return Some(ty);
            }
        }
        self.ty.get(name)
    }

    /// Gets a new [`GlobalPtr`] to a specified position in the current stack
    pub fn get_global_ptr(&self, ptr: Ptr) -> GlobalPtr {
        GlobalPtr::new(self.body.len() as u32, ptr)
    }
}
