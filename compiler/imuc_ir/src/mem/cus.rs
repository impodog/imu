use std::borrow::Cow;
use std::collections::BTreeSet;

/// A chunk of memory representing multiple elements
///
/// The order of the memory is not preserved
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Memory {
    Templ(crate::consts::TemplId),
    Bytes(super::Bytes),
    Elements(BTreeSet<Memory>),
}

impl Memory {
    /// Creates a memory representation of zero bytes
    pub fn zero_bytes() -> Self {
        Self::Bytes(super::Bytes::zero())
    }

    /// Creates a memory representation of empty elements
    pub fn elements() -> Self {
        Self::Elements(Default::default())
    }

    /// Pushes a memory into the representation, forming a custom type
    pub fn push(&mut self, memory: Memory) {
        match self {
            Self::Bytes(bytes) => {
                let set = BTreeSet::from_iter([Memory::Bytes(*bytes), memory]);
                *self = Memory::Elements(set)
            }
            Self::Templ(templ) => {
                let set = BTreeSet::from_iter([Memory::Templ(*templ), memory]);
                *self = Memory::Elements(set)
            }
            Self::Elements(elements) => {
                elements.insert(memory);
            }
        }
    }

    /// Assign a memory to each template of the template id
    pub fn instance(&mut self, id: crate::consts::TemplId, memory: Cow<Memory>) {
        match self {
            Self::Bytes(_) => {}
            Self::Templ(templ) => {
                if *templ == id {
                    *self = memory.into_owned();
                }
            }
            Self::Elements(elements) => {
                let mut vec = std::mem::take(elements).into_iter().collect::<Vec<_>>();
                vec.iter_mut().for_each(|element| {
                    element.instance(id, Cow::Borrowed(memory.as_ref()));
                });
                for element in vec.into_iter() {
                    elements.insert(element);
                }
            }
        }
    }
}
