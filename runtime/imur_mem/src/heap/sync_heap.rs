use crate::alloc::Memory;
use crate::ds::vec::Vec;
use crate::heap::HeapAlloc;
use crate::*;
use spin::RwLock;

/// A wrapper around the bare heap implementation,
/// providing thread safety around heap access using spin locks.
///
/// Spin locks provide fast access when there are few threads.
/// Although accesses are locked, actions that modify heap structure must lock the whole heap
/// (i.e. provide the methods with &mut self).
///
/// This must be initialized before use, or it will panic.
pub struct SyncHeap<'h, H: Memory + 'h> {
    heap: HeapPtr<'h, H>,
    node_locks: Vec<NodeLock, HeapPtr<'h, H>>,
    _pinned: core::marker::PhantomPinned,
}

/// A lock around a certain range of heap node.
struct NodeLock {
    start: usize,
    size: usize,
    lock: RwLock<()>,
}

unsafe impl<'h, H: Memory + 'h> Send for SyncHeap<'h, H> where H: Send {}
unsafe impl<'h, H: Memory + 'h> Sync for SyncHeap<'h, H> {}

impl<'h, H: Memory + 'h> HeapAlloc for SyncHeap<'h, H> {
    fn alloc(&mut self, size: usize) -> Option<usize> {
        let alloc_index = self.heap.alloc(size)?;
        // If the index is newly created
        #[allow(clippy::collapsible_if)]
        if self.node_locks.is_empty()
            || self
                .node_locks
                .access(self.node_locks.len() - 1, |node_lock| {
                    node_lock.start < alloc_index
                })
                .expect("last element should be in the vector")
        {
            if !self.push_lock(alloc_index) {
                return None;
            }
        }
        Some(alloc_index)
    }

    fn free(&mut self, alloc_index: usize) -> bool {
        self.heap.free(alloc_index)
    }

    fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize> {
        let new_alloc_index = self.heap.realloc(old_alloc_index, new_size)?;
        #[allow(clippy::collapsible_if)]
        if new_alloc_index != old_alloc_index {
            if !self.push_lock(new_alloc_index) {
                return None;
            }
        }
        Some(new_alloc_index)
    }

    unsafe fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        let lock_index = self.bsearch_lock(index)?;
        let lock_guard = self
            .node_locks
            .access(lock_index, |node_lock| node_lock.lock.read())
            .expect("lock index should be in bounds");
        let result = unsafe { self.heap.access(index, f) };
        core::mem::drop(lock_guard);
        result
    }

    unsafe fn access_mut<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        let lock_index = self.bsearch_lock(index)?;
        let lock_guard = self
            .node_locks
            .access(lock_index, |node_lock| node_lock.lock.write())
            .expect("lock index should be in bounds");
        let result = unsafe { self.heap.access_mut(index, f) };
        core::mem::drop(lock_guard);
        result
    }
}

impl<'h, H: Memory + 'h> SyncHeap<'h, H> {
    /// Creates a new sync heap with an external heap.
    pub fn new(heap: &'h crate::heap::bare_heap::Heap<H>) -> Self {
        let heap = HeapPtr::new(NonNull::from_ref(heap));
        Self {
            heap,
            node_locks: Vec::new(heap),
            _pinned: Default::default(),
        }
    }

    /// Initializes the heap, resolving self-reference and allocated internal heap free list.
    ///
    /// You must call this before any other actions, otherwise those methods will panic (implemented
    /// by internal heap).
    ///
    /// This only returns `false` if there is not enough space for initial locks.
    ///
    /// # Panics
    ///
    /// Panics if there is not enough memory for free list.
    pub fn init(&mut self) -> bool {
        let heap = unsafe { self.heap.as_mut() };
        heap.init();
        if !self.node_locks.reserve(4) {
            return false;
        }
        // This is not recursive as the node size is at least 4
        let alloc_index = self.node_locks.alloc_index();
        assert!(
            self.push_lock(alloc_index),
            "vector reserved space should be able to push one element"
        );
        true
    }

    /// Writes an element to a position on the heap.
    ///
    /// This function returns `None` if the index is not in any of the nodes,
    /// returns `false` if the index is out of bounds and does nothing,
    /// or returns `true`.
    ///
    /// # Safety
    ///
    /// You must guarantee type safety.
    /// `SyncHeap` now guarantees thread safety.
    pub unsafe fn copy<E>(&self, index: usize, value: &E) -> Option<bool> {
        let lock_index = self.bsearch_lock(index)?;
        let lock_guard = self
            .node_locks
            .access(lock_index, |node_lock| node_lock.lock.write())
            .expect("lock index should be in bounds");
        let mut ptr = self.heap.ptr;
        let result = unsafe { ptr.as_mut().copy(index, value) };
        core::mem::drop(lock_guard);
        Some(result)
    }

    /// Pushes a new lock to the back of the vector.
    ///
    /// This only returns `false` if there is not enough space for the vector.
    ///
    /// # Panics
    ///
    /// Panics if the alloc_index is out of bounds.
    #[must_use]
    fn push_lock(&mut self, alloc_index: usize) -> bool {
        let old_alloc_index = self.node_locks.alloc_index();
        let heap = unsafe { self.heap.as_mut() };
        let size = heap
            .size_of(alloc_index)
            .expect("alloc index should be in bounds");
        if !self.node_locks.push(NodeLock {
            start: alloc_index,
            size,
            lock: Default::default(),
        }) {
            return false;
        }
        let new_alloc_index = self.node_locks.alloc_index();
        if old_alloc_index != new_alloc_index {
            // This recursion only has depth 1, as increased length is always at least 4
            self.push_lock(new_alloc_index)
        } else {
            true
        }
    }

    /// Binary searches the lock that contains this index, returning the index in the vector.
    ///
    /// Returns `None` if the index is not in any of the lock ranges.
    fn bsearch_lock(&self, index: usize) -> Option<usize> {
        let mut left = 0;
        let mut right = self.node_locks.len();
        while left < right {
            let mid = (left + right) >> 1;
            // This is safe because the vector is hold inside the type and there is no race
            // conditions when &self is held.
            if self
                .node_locks
                .access(mid, |node_lock| node_lock.start <= mid)
                .expect("mid should be in vector ranges")
            {
                left = mid + 1;
            } else {
                right = mid;
            }
        }
        #[allow(clippy::if_same_then_else)]
        if left == self.node_locks.len() {
            None
        } else if self
            .node_locks
            .access(left, |node_lock| node_lock.start + node_lock.size <= index)
            .expect("result is checked to be inside vector ranges")
        {
            None
        } else {
            Some(left)
        }
    }
}

/// A pointer reference to `imur_mem::heap::Heap`.
/// This is highly unsafe, and you must guarantee that the heap outlives this pointer,
/// and that it is pinned to one pointer.
pub struct HeapPtr<'h, H: Memory + 'h> {
    ptr: NonNull<crate::heap::bare_heap::Heap<H>>,
    _phantom: core::marker::PhantomData<&'h H>,
}
impl<'h, H: Memory> Clone for HeapPtr<'h, H> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'h, H: Memory> Copy for HeapPtr<'h, H> {}

impl<'h, H: Memory + 'h> HeapPtr<'h, H> {
    /// Creates a new pointer to the heap. This type is highly unsafe, and you must ensure that the
    /// heap outlives anywhere that uses this.
    pub fn new(ptr: NonNull<crate::heap::bare_heap::Heap<H>>) -> Self {
        Self {
            ptr,
            _phantom: Default::default(),
        }
    }

    /// Converts the internal pointer to an immutable reference with a panic message if it is null.
    ///
    /// # Safety
    ///
    /// The pointer must not be dangling.
    pub unsafe fn as_ref(self) -> &'h crate::heap::bare_heap::Heap<H> {
        unsafe { self.ptr.as_ref() }
    }

    /// Converts the internal pointer to a mutable reference with a panic message if it is null.
    ///
    /// # Safety
    ///
    /// The pointer must not be dangling.
    pub unsafe fn as_mut(mut self) -> &'h mut crate::heap::bare_heap::Heap<H> {
        unsafe { self.ptr.as_mut() }
    }
}

impl<'h, H: Memory> HeapAlloc for HeapPtr<'h, H> {
    fn alloc(&mut self, size: usize) -> Option<usize> {
        unsafe { self.as_mut().alloc(size) }
    }

    fn free(&mut self, alloc_index: usize) -> bool {
        unsafe { self.as_mut().free(alloc_index) }
    }

    fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize> {
        unsafe { self.as_mut().realloc(old_alloc_index, new_size) }
    }

    unsafe fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        unsafe { self.as_ref().access(index, f) }
    }

    unsafe fn access_mut<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        // NOTE: This is unsafe because an immutable reference is taken as mutable.
        // Only use this when you mutably borrow the pointed heap!
        unsafe { self.as_ref().access(index, f) }
    }
}
