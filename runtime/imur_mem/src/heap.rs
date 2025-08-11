use crate::*;
use alloc::Memory;

/// A linked list containing pointers to pre-allocated chunks of the heap.
/// The actual heap allocation begins at the pointer to this node + sizeof(Node),
/// and it starts with `Meta`.
#[repr(C)]
struct Node {
    /// Handles the linked list, may be NULL of the last node
    next: *mut Node,
}

/// Meta data stored before heap allocations with a fixed size that marks:
/// `size`: Size of the chunk
#[repr(C, align(8))]
// FIXME: Do we change alignment on 32 bits?
struct Meta {
    size: u8,
}

/// An IMU heap that operates on a linear adapter to `crate::alloc::Memory`,
/// providing efficient heap allocations.
pub struct Heap<M: Memory> {
    mem: M,
    /// Points to a vector of linked lists containing nodes of different sizes.
    free_list: *const *mut Node,
    max_block: u8,
}

impl Node {
    const SIZE: usize = core::mem::size_of::<Self>();
}

impl Meta {
    const SIZE: usize = core::mem::size_of::<Self>();
}

impl<M: Memory> Heap<M> {
    /// Initializes the free list which contains no nodes
    ///
    /// # Panics
    ///
    /// Panics if free list allocation failed
    pub fn init(mut self) -> Self {
        let free_list_size = core::mem::size_of::<*mut Node>() * self.max_block as usize;
        if self.mem.grow_more(free_list_size) {
            panic!("not enough memory for heap free list");
        }
        self.free_list = self
            .mem
            .access(0, free_list_size)
            .expect("should be valid after allocation")
            .cast()
            .as_ptr();
        self
    }

    /// Creates a new heap allocator with given memory.
    /// Note that this is uninitialized, and to use it, you must first call `Self::init`.
    pub fn new(mem: M, max_block: u8) -> Self {
        Self {
            mem,
            free_list: core::ptr::null(),
            max_block,
        }
        .init()
    }

    /// Tests if free list if null, and panics if so.
    fn assert_init(&self) {
        if self.free_list.is_null() {
            panic!("free list is uninitialized");
        }
    }

    /// Creates a new allocation node with given size in binary power.
    /// This does not push it in the free list, and causes memory leak if not handled.
    ///
    /// # Panics
    ///
    /// Panics if the size is too big(either not enough memory or the size overflows usize).
    fn make_node(&self, size: u8) -> *mut Node {
        let alloc_size = 1usize
            .checked_shl(size as _)
            .expect("size should be lower than usize");
        let pad = ALIGNMENT - alloc_size % ALIGNMENT;
        let total = Node::SIZE + Meta::SIZE + alloc_size + pad;
        // TODO: Implement make_node
    }
}
