/// Trait that defines the heap interface. This makes it possible to implement custom heaps, add
/// locks to the current heap, etc..
pub trait HeapAlloc {
    /// Allocates memory of size, returning its unique index, which can be used to further operate on this
    /// allocation chunk. Same as C alloc, memory will not free until `Self::free` is called.
    ///
    /// Returns `None` only if allocation is not possible, such as the size overflows the memory.
    fn alloc(&mut self, size: usize) -> Option<usize>;

    /// Frees the memory chunk by the index that `Self::alloc` returns.
    /// Returns whether freeing is successful. This only fails because the index does not point to
    /// a chunk.
    ///
    /// Freeing the same node multiple times is considered successful and does nothing, but
    /// may cause unexpected behavior as the node may be used by other code.
    #[must_use]
    fn free(&mut self, alloc_index: usize) -> bool;

    /// Reallocates a previously allocated chunk, to extend or shrink the chunk,
    /// while preserving the most data of the original chunk.
    ///
    /// This may change its index, so a new index is returned.
    /// After calling this, the old index must not be used anymore.
    ///
    /// Returns `None` only if allocation is not possible, such as the size overflows the memory.
    #[must_use]
    fn realloc(&mut self, old_alloc_index: usize, new_size: usize) -> Option<usize>;

    /// Reads the heap with an action on the element.
    ///
    /// This function returns `None` if the index is out of bounds and does nothing,
    /// or it returns the result of the function provided.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    unsafe fn access<F, E, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&E) -> R,
        E: Sized;

    /// Writes the heap with an action on the element.
    ///
    /// This function returns `None` if the index is out of bounds and does nothing,
    /// or it returns the result of the function provided.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    unsafe fn access_mut<F, E, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut E) -> R,
        E: Sized;

    /// Writes an element to a position on the heap.
    ///
    /// This function returns `false` if the index is out of bounds and does nothing.
    ///
    /// # Safety
    ///
    /// The index must be inside a valid allocation, and you must guarantee type safety.
    #[must_use]
    unsafe fn copy<E>(&mut self, index: usize, value: &E) -> bool;
}
