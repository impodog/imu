use crate::*;
use heap::HeapAlloc;

/// An IMU dynamic array based on a heap.
///
/// # Note
///
/// This forces alignment to pointer, to ensure quick queries. However this may waste a lot of
/// memory for smaller types, such as `u8`. In this case you need to group u8 together, then use
/// this vector.
pub struct Vec<E, H: HeapAlloc> {
    heap: H,
    len: usize,
    capa: usize,
    alloc_index: usize,
    _phantom: core::marker::PhantomData<E>,
}

impl<E, H: HeapAlloc> Vec<E, H> {
    const ELEM_SIZE: usize = mem::size_of::<E>();
    const ELEM_ALIGNED_SIZE: usize = {
        let size = mem::size_of::<E>();
        let align = size % ALIGNMENT;
        if align == 0 {
            size
        } else {
            size + (ALIGNMENT - align)
        }
    };

    /// Creates a new empty vector.
    pub fn new(heap: H) -> Self {
        Self {
            heap,
            len: 0,
            // Marks that the vector is uninitialized
            capa: 0,
            // This is uninitialized and never used
            alloc_index: 0,
            _phantom: Default::default(),
        }
    }

    /// Returns if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the length of the vector.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Pushes the element into the vector, returning if the operation is successful.
    ///
    /// Time complexity is worst case O(n) (when reallocating), amortized O(1).
    ///
    /// This only fails because of internal heap error. This does not panic.
    pub fn push(&mut self, elem: E) -> bool {
        if (self.len + 1) * Self::ELEM_ALIGNED_SIZE > self.capa {
            if self.capa == 0 {
            } else {
                let new_size = self.capa * 2;
                if let Some(new_alloc_index) = self.heap.realloc(self.alloc_index, new_size) {
                    self.alloc_index = new_alloc_index;
                } else {
                    return false;
                }
            };
        }
        unsafe {
            if !self
                .heap
                .copy(self.alloc_index + self.len * Self::ELEM_ALIGNED_SIZE, &elem)
            {
                return false;
            }
        }
        // Forget to prevent reusing `elem`
        mem::forget(elem);
        self.len += 1;
        true
    }

    /// Pops an element from the vector, returning it back to the caller, if not empty.
    ///
    /// Time complexity is worst case O(1).
    ///
    /// This does not shrink the heap allocation.
    ///
    /// # Panics
    ///
    /// Panics if internal heap fails to access the previously allocated memory, which is against
    /// heap properties.
    pub fn pop(&mut self) -> Option<E> {
        if self.is_empty() {
            None
        } else {
            let elem = unsafe {
                self.heap
                    .access(
                        self.alloc_index + (self.len - 1) * Self::ELEM_ALIGNED_SIZE,
                        |elem: &E| {
                            let mut buffer = mem::MaybeUninit::<E>::uninit();
                            ptr::copy_nonoverlapping(
                                elem as *const E as *const u8,
                                buffer.as_mut_ptr().cast::<u8>(),
                                Self::ELEM_SIZE,
                            );
                            buffer.assume_init()
                        },
                    )
                    .expect("should succeed to access previously allocated memory")
            };
            self.len -= 1;
            Some(elem)
        }
    }

    /// Uses the provided function to access the element at index, immutably.
    /// To access mutably use `Self::access_mut`.
    ///
    /// Returns `None` if index is out of bounds, or return the provided function's result wrapped
    /// in `Some`.
    ///
    /// Returning the reference is not possible, because heap borrows can be implemented under locks,
    /// and the vector does not uniquely own the heap.
    ///
    /// # Panics
    ///
    /// Panics if internal heap fails to access the previously allocated memory, which is against
    /// heap properties.
    pub fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&E) -> R,
    {
        if index >= self.len {
            None
        } else {
            let result = unsafe {
                self.heap
                    .access(self.alloc_index + index * Self::ELEM_ALIGNED_SIZE, f)
                    .expect("should succeed to access previously allocated memory")
            };
            Some(result)
        }
    }

    /// Uses the provided function to access the element at index, mutably.
    /// To access immutably use `Self::access`.
    ///
    /// Returns `None` if index is out of bounds, or return the provided function's result wrapped
    /// in `Some`.
    ///
    /// Returning the reference is not possible, because heap borrows can be implemented under locks,
    /// and the vector does not uniquely own the heap.
    ///
    /// # Panics
    ///
    /// Panics if internal heap fails to access the previously allocated memory, which is against
    /// heap properties.
    pub fn access_mut<F, R>(&mut self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut E) -> R,
    {
        if index >= self.len {
            None
        } else {
            let result = unsafe {
                self.heap
                    .access_mut(self.alloc_index + index * Self::ELEM_ALIGNED_SIZE, f)
                    .expect("should succeed to access previously allocated memory")
            };
            Some(result)
        }
    }

    /// Shrinks allocation to the minimum that can hold current data.
    ///
    /// This only returns `false` if internal allocation fails. In this case, no action is
    /// performed.
    ///
    /// You should only call this when you don't push elements anymore and want to keep the vector
    /// for a long time.
    pub fn shrink_to_fit(&mut self) -> bool {
        if self.len == 0 && self.capa != 0 {
            // When there are no elements, free the whole memory.
            if !self.heap.free(self.alloc_index) {
                return false;
            }
            self.alloc_index = 0;
            self.capa = 0;
            true
        } else {
            // The last element does not require padding.
            let shrunk_size = (self.len - 1) * Self::ELEM_ALIGNED_SIZE + Self::ELEM_SIZE;
            if let Some(new_alloc_index) = self.heap.realloc(self.alloc_index, shrunk_size) {
                self.alloc_index = new_alloc_index;
                self.capa = shrunk_size;
                true
            } else {
                false
            }
        }
    }
}
