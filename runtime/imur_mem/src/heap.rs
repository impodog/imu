use crate::*;
use alloc::Memory;

/// A linked list containing pointers to pre-allocated chunks of the heap.
/// The actual heap allocation begins at the pointer to this node + sizeof(Node), and it starts with `Meta`.
#[repr(C)]
struct Node {
    /// Handles the linked list, may be `usize::MAX` of the last node
    next: usize,
}

/// Meta data stored before heap allocations with a fixed size that marks:
/// `size`: Size of the chunk
/// `magic`: The magic number, used for problems
#[repr(C, align(8))]
#[derive(Clone, Copy)]
// FIXME: Do we change alignment on 32 bits?
struct Meta {
    freed: bool,
    block_size: u8,
    magic: u32,
}

/// An IMU heap that operates on a linear adapter to `crate::alloc::Memory`,
/// providing efficient heap allocations.
///
/// The heap is organized in this way using linear memory(ignore padding):
/// free list | [node/meta/alloc] [node/meta/alloc] ...
/// where free list contains the head node of each block size, which are all powers of 2.
pub struct Heap<M: Memory> {
    mem: M,
    max_block: u8,
    top: usize,
    init: bool,
}

impl Node {
    const SIZE: usize = mem::size_of::<Self>();
}

impl Meta {
    const SIZE: usize = mem::size_of::<Self>();
    const MAGIC_NUMBER: u32 = 0xCAFCAC03;
}

impl<M: Memory> Heap<M> {
    /// Initializes the free list which contains no nodes.
    ///
    /// # Panics
    ///
    /// Panics if free list allocation failed.
    pub fn init(&mut self) {
        let free_list_size = mem::size_of::<usize>() * self.max_block as usize;
        if !self.mem.grow_more(free_list_size) {
            panic!("not enough memory for heap free list");
        }
        for block_size in 0..self.max_block {
            let head_index = self.head_index(block_size);
            let head_ptr = self
                .mem
                .access_mut(head_index, Node::SIZE)
                .expect("should exist because block_size is in bounds");
            unsafe {
                *head_ptr.cast::<usize>().as_ptr() = usize::MAX;
            }
        }
        self.top += free_list_size;
        self.init = true;
    }

    /// Creates a new heap allocator with given memory.
    /// Note that this is uninitialized, and to use it, you must first call `Self::init`.
    ///
    /// `max_block` is exclusive, meaning only block sizes less than that is accepted.
    ///
    /// # Panics
    ///
    /// Panics if 2 ^ (`max_block` - 1) overflows `isize`.
    pub fn new(mem: M, max_block: u8) -> Self {
        // NOTE: `max_block` overflowing isize is the same as `max_block` - 1 overflowing usize
        1usize
            .checked_shl(max_block as _)
            .expect("2 ^ max_block should not overflow usize");
        Self {
            mem,
            max_block,
            top: 0,
            init: false,
        }
    }

    /// Allocates memory of size, returning its unique index, which can be used to further operate on this
    /// allocation chunk. Same as C alloc, memory will not free until `Self::free` is called.
    ///
    /// Returns `None` if size is greater than 2 ^ (`Self::max_block` - 1) or is 0, or if internal
    /// memory allocation failed.
    #[must_use]
    pub fn alloc(&mut self, size: usize) -> Option<usize> {
        let block_size = size.next_power_of_two().ilog2() as u8;
        if block_size == 0 || block_size >= self.max_block {
            return None;
        }
        #[allow(clippy::manual_map)]
        if let Some(node_index) = self.pop_node(block_size) {
            Some(node_index + Node::SIZE + Meta::SIZE)
        } else if let Some(node_index) = self.make_node(block_size) {
            Some(node_index + Node::SIZE + Meta::SIZE)
        } else {
            None
        }
    }

    /// Frees the memory chunk by the index that `Self::alloc` returns.
    /// Returns whether freeing is successful. This only fails because the index does not point to
    /// a chunk.
    ///
    /// Freeing the same node multiple times is considered successful and does nothing, but
    /// may cause unexpected behavior as the node may be used by other code.
    #[must_use]
    pub fn free(&mut self, alloc_index: usize) -> bool {
        if let Some(node_index) = alloc_index.checked_sub(Node::SIZE + Meta::SIZE) {
            self.push_node(node_index)
        } else {
            false
        }
    }

    /// Reallocates a previously allocated chunk, to extend or shrink the chunk,
    /// while preserving the most of the original chunk.
    ///
    /// This may change its index, so a new index is returned.
    ///
    /// After calling this, the old index must not be used anymore.
    ///
    /// Returns `None` if size is greater than 2 ^ (`Self::max_block` - 1) or is 0, if internal
    /// memory allocation failed, or if the give index does not point to a valid chunk.
    /// Otherwise return the new allocation index.
    #[must_use]
    pub fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize> {
        if let Some(old_node_index) = old_alloc_index.checked_sub(Node::SIZE + Meta::SIZE) {
            let new_block_size = new_size.next_power_of_two().ilog2() as u8;
            if new_block_size == 0 || new_block_size >= self.max_block {
                return None;
            }
            let meta = self.copy_meta(old_node_index)?;
            if meta.freed {
                return None;
            }
            if meta.block_size == new_block_size {
                Some(old_alloc_index)
            } else {
                let new_alloc_index = self.alloc(new_size)?;
                let old_size = 1usize << meta.block_size;
                unsafe {
                    let old_alloc_ptr = self
                        .mem
                        .access_mut(old_alloc_index, old_size)
                        .expect("old chunk should be valid");
                    let new_alloc_ptr = self
                        .mem
                        .access_mut(new_alloc_index, new_size)
                        .expect("old chunk should be valid");
                    ptr::copy_nonoverlapping(
                        old_alloc_ptr.cast::<u8>().as_ptr(),
                        new_alloc_ptr.cast::<u8>().as_ptr(),
                        old_size.min(new_size),
                    );
                }
                assert!(self.free(old_alloc_index));
                Some(new_alloc_index)
            }
        } else {
            None
        }
    }

    /// Reads the heap with an action on the element.
    ///
    /// This function returns `None` if the index is out of bounds and does nothing,
    /// or it returns the result of the function provided.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    pub unsafe fn access<F, E, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&E) -> R,
        E: Sized,
    {
        let size = mem::size_of::<E>();
        #[allow(clippy::manual_map)]
        if let Some(ptr) = self.mem.access(index, size) {
            Some(f(unsafe { ptr.cast::<E>().as_ref() }))
        } else {
            None
        }
    }

    /// Writes the heap with an action on the element.
    ///
    /// This function returns `None` if the index is out of bounds and does nothing,
    /// or it returns the result of the function provided.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    pub unsafe fn access_mut<F, E, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut E) -> R,
        E: Sized,
    {
        let size = mem::size_of::<E>();
        #[allow(clippy::manual_map)]
        if let Some(ptr) = self.mem.access_mut(index, size) {
            Some(f(unsafe { ptr.cast::<E>().as_mut() }))
        } else {
            None
        }
    }

    /// Writes an element to a position on the heap.
    ///
    /// This function returns `None` if the index is out of bounds and does nothing.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    pub unsafe fn copy<E>(&mut self, index: usize, value: &E) -> bool {
        let size = mem::size_of::<E>();
        #[allow(clippy::manual_map)]
        if let Some(ptr) = self.mem.access_mut(index, size) {
            unsafe {
                ptr::copy_nonoverlapping(
                    value as *const E as *const u8,
                    ptr.as_ptr() as *mut u8,
                    size,
                );
            }
            true
        } else {
            false
        }
    }

    /// Supports stack-like operation where memory is extended by `Self::top`.
    /// This does not change `Self::top`, but only calls `Self::mem`;
    /// Returns whether allocation is successful.
    #[must_use]
    fn alloc_least(&mut self, size: usize) -> bool {
        let diff = self.mem.size() - self.top;
        if diff < size {
            self.mem.grow_more(size - diff)
        } else {
            true
        }
    }

    /// Gets the head node index stored in the free list, or return `usize::MAX` if the list is empty.
    ///
    /// # Panics
    ///
    /// Panics if the block size is not less than `Self::max_block`
    fn head_index(&self, block_size: u8) -> usize {
        assert!(
            block_size < self.max_block,
            "block size should be less than Self::max_block"
        );
        mem::size_of::<usize>() * block_size as usize
    }

    /// Copies the meta data from the given node.
    /// Returns `None` if the given index is out of bounds, or if it is not a node index.
    fn copy_meta(&self, node_index: usize) -> Option<Meta> {
        let meta_index = node_index + Node::SIZE;
        if let Some(meta_ptr) = self.mem.access(meta_index, Meta::SIZE) {
            let meta = unsafe { meta_ptr.cast::<Meta>().as_ref() };
            if meta.magic != Meta::MAGIC_NUMBER {
                None
            } else {
                Some(*meta)
            }
        } else {
            None
        }
    }

    /// Attempts to pop an allocation node from free list.
    /// Returns the memory index to the node, or `None` if the list is empty.
    ///
    /// This also sets its freed flag to `false`.
    ///
    /// # Panics
    ///
    /// Panics if the size is not less than `Self::max_block`.
    fn pop_node(&mut self, block_size: u8) -> Option<usize> {
        assert!(
            block_size < self.max_block,
            "block size should be less than Self::max_block"
        );
        let head_index = self.head_index(block_size);
        let head_ptr = self
            .mem
            .access_mut(head_index, mem::size_of::<usize>())
            .expect("should be in the free list")
            .cast::<usize>();
        let node_index = unsafe { *head_ptr.as_ptr() };
        if node_index == usize::MAX {
            None
        } else {
            let node_ptr = self
                .mem
                .access_mut(node_index, Node::SIZE)
                .expect("should be a valid node in the heap")
                .cast::<Node>();
            let next_index = unsafe { node_ptr.as_ref().next };
            unsafe {
                *head_ptr.as_ptr() = next_index;
            }
            let meta_index = node_index + Node::SIZE;
            let meta_ptr = self
                .mem
                .access_mut(meta_index, Meta::SIZE)
                .expect("should be a valid node on the heap");
            unsafe {
                meta_ptr.cast::<Meta>().as_mut().freed = false;
            }
            Some(node_index)
        }
    }

    /// Pushes the allocation node into the free list.
    /// Returns whether this node is a proper node, tested by the magic number and memory bounds.
    ///
    /// If the node is not a proper node or its freed flag is already `true`, no action is done.
    ///
    /// This also sets its freed flag to `true`.
    #[must_use]
    fn push_node(&mut self, node_index: usize) -> bool {
        let meta_index = node_index + Node::SIZE;
        let block_size = if let Some(meta_ptr) = self.mem.access_mut(meta_index, Meta::SIZE) {
            let meta = unsafe { meta_ptr.cast::<Meta>().as_mut() };
            if meta.magic != Meta::MAGIC_NUMBER {
                return false;
            } else if meta.freed {
                return true;
            } else {
                meta.freed = true;
                meta.block_size
            }
        } else {
            return false;
        };
        let head_index = self.head_index(block_size);
        let mut head_ptr = self
            .mem
            .access_mut(head_index, mem::size_of::<usize>())
            .expect("should be in the free list")
            .cast::<usize>();
        let next = unsafe { mem::replace(head_ptr.as_mut(), node_index) };
        let mut node_ptr = self
            .mem
            .access_mut(node_index, Node::SIZE)
            .expect("should be in bounds after checking")
            .cast::<Node>();
        unsafe {
            node_ptr.as_mut().next = next;
        }
        true
    }

    /// Creates a new allocation node with given size in binary power.
    /// This does not push it in the free list, and causes memory leak if not handled.
    ///
    /// This also sets its freed flag to `false`.
    ///
    /// Returns `None` if there is no sufficient memory, or return the node index.
    ///
    /// # Panics
    ///
    /// Panics if the size is not less than `Self::max_block`.
    #[must_use]
    fn make_node(&mut self, block_size: u8) -> Option<usize> {
        assert!(
            block_size < self.max_block,
            "block size should be less than Self::max_block"
        );
        let alloc_size = 1usize << block_size;
        // Note: padding ensures the next node/meta/alloc
        let pad = ALIGNMENT - alloc_size % ALIGNMENT;
        let total = Node::SIZE + Meta::SIZE + alloc_size + pad;
        if !self.alloc_least(total) {
            return None;
        }

        let node_index = self.top;
        self.top += Node::SIZE;
        let meta_index = self.top;
        self.top += Meta::SIZE;
        let _alloc_index = self.top;
        self.top += alloc_size + pad;

        let meta_ptr = self
            .mem
            .access_mut(meta_index, Meta::SIZE)
            .expect("should contain meta memory after initialization");
        unsafe {
            let meta = meta_ptr.cast::<Meta>().as_mut();
            meta.block_size = block_size;
            meta.freed = false;
            meta.magic = Meta::MAGIC_NUMBER;
        }

        Some(node_index)
    }
}
