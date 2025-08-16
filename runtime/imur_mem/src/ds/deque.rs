use crate::*;
use heap::HeapAlloc;

/// An IMU dynamic deque based on a heap.
///
/// # Note
///
/// This forces alignment to pointer, to ensure quick queries. However this may waste a lot of
/// memory for smaller types, such as `u8`. In this case you need to group u8 together, then use
/// this deque.
pub struct Deque<E, H: HeapAlloc> {
    heap: H,
    head: usize,
    tail: usize,
    is_empty: bool,
    capa: usize,
    alloc_index: usize,
    _phantom: core::marker::PhantomData<E>,
}

impl<E, H: HeapAlloc> Deque<E, H> {
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

    /// Creates a new empty deque.
    pub fn new(heap: H) -> Self {
        Self {
            heap,
            head: 0,
            tail: 0,
            // Marks that the deque is empty, to distinguish full and empty when head == tail
            is_empty: true,
            // Marks that the deque is uninitialized, note that the capacity is defined
            // differently from vector: it is the number of elements instead of size.
            capa: 0,
            // This is uninitialized and never used
            alloc_index: 0,
            _phantom: Default::default(),
        }
    }

    /// Returns if the deque is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// Returns if the deque has used all of its current capacity.
    #[inline]
    fn is_full(&self) -> bool {
        !self.is_empty && self.head == self.tail
    }

    /// Returns the length of the deque.
    pub fn len(&self) -> usize {
        if self.is_empty {
            0
        } else if self.head < self.tail {
            self.tail - self.head
        } else {
            self.tail + self.capa - self.head
        }
    }

    /// Appends an element to the back of the deque, returning if the operation is successful.
    ///
    /// Time complexity is worst case O(n) (when reallocating), amortized O(1).
    ///
    /// This only fails because of internal heap error.
    pub fn push_back(&mut self, elem: E) -> bool {
        if !self.extend_when_full() {
            return false;
        }
        unsafe {
            self.push_at(self.tail, elem);
        }
        self.tail += 1;
        // Jump to front immediately, to allow the next push_back to be done at 0.
        if self.tail == self.capa {
            self.tail = 0;
        }
        self.is_empty = false;
        true
    }

    /// Appends an element to the front of the deque, returning if the operation is successful.
    ///
    /// Time complexity is worst case O(n) (when reallocating), amortized O(1).
    ///
    /// This only fails because of internal heap error.
    pub fn push_front(&mut self, elem: E) -> bool {
        // Small optimization
        if self.is_empty {
            return self.push_back(elem);
        }
        if !self.extend_when_full() {
            return false;
        }
        // Because self.head is inclusive, we need to operate on head first, then push.
        if self.head == 0 {
            self.head = self.capa - 1;
        }
        unsafe {
            self.push_at(self.head, elem);
        }
        self.is_empty = false;
        true
    }

    /// Pops an element from the back of the deque, returning it back to the caller, if not empty.
    ///
    /// Time complexity is worst case O(1).
    ///
    /// This does not shrink the heap allocation.
    pub fn pop_back(&mut self) -> Option<E> {
        if self.is_empty() {
            None
        } else {
            let back = if self.tail == 0 {
                self.capa - 1
            } else {
                self.tail - 1
            };
            let elem = unsafe {
                self.heap
                    .access(self.alloc_index + back * Self::ELEM_ALIGNED_SIZE, |ptr| {
                        let mut buffer = mem::MaybeUninit::<E>::uninit();
                        ptr::copy_nonoverlapping(
                            ptr.cast::<u8>().as_ptr(),
                            buffer.as_mut_ptr().cast::<u8>(),
                            Self::ELEM_SIZE,
                        );
                        buffer.assume_init()
                    })
                    .expect("should succeed to access previously allocated memory")
            };
            self.tail = back;
            if self.head == self.tail {
                self.is_empty = true;
            }
            Some(elem)
        }
    }

    /// Pops an element from the front of the deque, returning it back to the caller, if not empty.
    ///
    /// Time complexity is worst case O(1).
    ///
    /// This does not shrink the heap allocation.
    pub fn pop_front(&mut self) -> Option<E> {
        if self.is_empty() {
            None
        } else {
            let elem = unsafe {
                self.heap
                    .access(
                        self.alloc_index + self.head * Self::ELEM_ALIGNED_SIZE,
                        |ptr| {
                            let mut buffer = mem::MaybeUninit::<E>::uninit();
                            ptr::copy_nonoverlapping(
                                ptr.cast::<u8>().as_ptr(),
                                buffer.as_mut_ptr().cast::<u8>(),
                                Self::ELEM_SIZE,
                            );
                            buffer.assume_init()
                        },
                    )
                    .expect("should succeed to access previously allocated memory")
            };
            self.head += 1;
            if self.head == self.capa {
                self.head = 0;
            }
            if self.head == self.tail {
                self.is_empty = true;
            }
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
    pub fn access<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&E) -> R,
    {
        if index >= self.len() {
            None
        } else {
            let abs_index = if index + self.head > self.capa {
                index + self.head - self.capa
            } else {
                index + self.head
            };
            let result = unsafe {
                self.heap
                    .access(
                        self.alloc_index + abs_index * Self::ELEM_ALIGNED_SIZE,
                        |ptr| f(ptr.cast::<E>().as_ref()),
                    )
                    .expect("should succeed to access previously allocated memory")
            };
            Some(result)
        }
    }

    /// Uses the provided function to access the element at index relative to deque head, mutably.
    /// To access immutably use `Self::access`.
    ///
    /// Returns `None` if index is out of bounds, or return the provided function's result wrapped
    /// in `Some`.
    ///
    /// Returning the reference is not possible, because heap borrows can be implemented under locks,
    /// and the vector does not uniquely own the heap.
    pub fn access_mut<F, R>(&self, index: usize, f: F) -> Option<R>
    where
        F: FnOnce(&mut E) -> R,
    {
        if index >= self.len() {
            None
        } else {
            let abs_index = if index + self.head > self.capa {
                index + self.head - self.capa
            } else {
                index + self.head
            };
            let result = unsafe {
                self.heap
                    .access_mut(
                        self.alloc_index + abs_index * Self::ELEM_ALIGNED_SIZE,
                        |ptr| f(ptr.cast::<E>().as_mut()),
                    )
                    .expect("should succeed to access previously allocated memory")
            };
            Some(result)
        }
    }

    /// When the deque is full, extends the allocation.
    ///
    /// This only returns false when allocation fails.
    fn extend_when_full(&mut self) -> bool {
        if self.is_full() {
            if self.capa == 0 {
                if let Some(alloc_index) = self.heap.alloc(Self::ELEM_ALIGNED_SIZE) {
                    self.capa = 1;
                    self.alloc_index = alloc_index;
                } else {
                    return false;
                }
            } else {
                let new_capa = self.capa * 2;
                // Manually implement realloc for buffer copying
                if let Some(new_alloc_index) = self.heap.alloc(new_capa * Self::ELEM_ALIGNED_SIZE) {
                    if !self.is_empty {
                        unsafe {
                            self.heap
                                .access_mut(new_alloc_index, |new_alloc_ptr| {
                                    self.heap
                                        .access(self.alloc_index, |old_alloc_ptr| {
                                            if self.head < self.tail {
                                                let size = (self.tail - self.head)
                                                    * Self::ELEM_ALIGNED_SIZE;
                                                ptr::copy_nonoverlapping(
                                                    old_alloc_ptr
                                                        .byte_add(
                                                            self.head * Self::ELEM_ALIGNED_SIZE,
                                                        )
                                                        .as_ptr(),
                                                    new_alloc_ptr.as_ptr(),
                                                    size,
                                                );
                                                self.tail -= self.head;
                                                self.head = 0;
                                            } else {
                                                // First copy bytes after head
                                                let size_head = (self.capa - self.head)
                                                    * Self::ELEM_ALIGNED_SIZE;
                                                ptr::copy_nonoverlapping(
                                                    old_alloc_ptr
                                                        .byte_add(
                                                            self.head * Self::ELEM_ALIGNED_SIZE,
                                                        )
                                                        .as_ptr(),
                                                    new_alloc_ptr.as_ptr(),
                                                    size_head,
                                                );
                                                // Then copy bytes before tail
                                                let size_tail = self.tail * Self::ELEM_ALIGNED_SIZE;
                                                ptr::copy_nonoverlapping(
                                                    old_alloc_ptr.as_ptr(),
                                                    new_alloc_ptr.as_ptr().byte_add(size_head),
                                                    size_tail,
                                                );
                                                self.tail = self.tail + self.capa - self.head;
                                                self.head = 0;
                                            }
                                        })
                                        .expect("old allocation should be in bounds")
                                })
                                .expect("new allocation should be in bounds");
                        }
                    } else {
                        self.head = 0;
                        self.tail = 0;
                    }

                    assert!(
                        self.heap.free(self.alloc_index),
                        "should succeed to free previously allocated memory"
                    );

                    self.alloc_index = new_alloc_index;
                } else {
                    return false;
                }
            };
        }
        true
    }

    /// Inserts an element at the index relative to allocation begin.
    ///
    /// # Safety
    ///
    /// The given index must be in allocation bounds.
    unsafe fn push_at(&mut self, index: usize, elem: E) {
        unsafe {
            self.heap
                .access_mut(self.alloc_index + index * Self::ELEM_ALIGNED_SIZE, |ptr| {
                    ptr::copy_nonoverlapping(
                        &elem as *const E as *const u8,
                        ptr.cast::<u8>().as_ptr(),
                        Self::ELEM_SIZE,
                    );
                })
                .expect("should succeed to access previously allocated memory")
        }
        mem::forget(elem);
    }
}
