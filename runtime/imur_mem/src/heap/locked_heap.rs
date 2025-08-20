use crate::heap::HeapAlloc;
use crate::*;
use spin::RwLock;

/// A heap wrapper around `HeapAlloc` types, adding a simple lock for mutable heap accesses,
/// allowing copying its reference.
///
/// This only locks the whole heap, while internal node locks are implemented by `SyncHeap` in this
/// crate, allowing `HeapAlloc::access_mut`.
///
/// Note that its reference implements `HeapAlloc`, rather than the type itself.
pub struct LockedHeap<H: HeapAlloc> {
    lock: RwLock<H>,
}

impl<H: HeapAlloc> LockedHeap<H> {
    /// Wraps the inner heap with the lock.
    pub fn new(heap: H) -> Self {
        Self {
            lock: RwLock::new(heap),
        }
    }
}

impl<H: HeapAlloc> HeapAlloc for &LockedHeap<H> {
    fn alloc(&mut self, size: usize) -> Option<usize> {
        self.lock.write().alloc(size)
    }
    fn free(&mut self, alloc_index: usize) -> bool {
        self.lock.write().free(alloc_index)
    }
    fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize> {
        self.lock.write().realloc(old_alloc_index, new_size)
    }
    unsafe fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        unsafe { self.lock.read().access(index, f) }
    }
    unsafe fn access_mut<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(NonNull<()>) -> R,
    {
        unsafe { self.lock.read().access(index, f) }
    }
}
