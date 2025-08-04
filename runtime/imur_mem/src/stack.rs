use crate::alloc::Memory;
use crate::*;

/// An IMU stack that manages its internal memory, supporting typical stack operations.
/// Please note that the stack does not support variability after the element is inserted.
pub struct Stack<T: Memory> {
    mem: T,
    top: usize,
}

impl<T: Memory> Stack<T> {
    /// Creates an empty stack holding a memory
    pub fn new(mem: T) -> Self {
        Self { mem, top: 0 }
    }

    /// Pushes an element of given size into the stack, returning its offset,
    /// which can be used to access that element later.
    /// If allocation fails by the internal [`Memory`], [`None`] is returned.
    pub fn push<E>(&mut self, elem: E) -> Option<usize>
    where
        E: Sized,
    {
        let size: usize = core::mem::size_of::<E>();
        let rem = self.mem.size() - self.top;
        if rem < size {
            let extra = size - rem;
            if !self.mem.grow_more(extra) {
                return None;
            }
        }

        let vptr = self.top;
        // Stores the value
        let ptr = self
            .mem
            .access_mut(vptr, size)
            .expect("Should contain the stack element");
        unsafe {
            *(ptr.as_ptr() as *mut E) = elem;
        };
        self.top += size;
        Some(vptr)
    }

    /// Resets the stack size to the given argument. This function only shrinks, so if you give a
    /// size bigger than the current size, no action will be done.
    pub fn reset(&mut self, size: usize) {
        if size < self.top {
            self.top = size;
            self.mem.shrink(size);
        }
    }

    /// Accesses the stack for the element at pointer, or return [`None`] if the pointer is out of
    /// bounds, or the size of the element would exceed to size limits.
    ///
    /// # Panics
    /// - Panics when the pointer is misaligned(i.e. [`crate::ALIGNMENT`] does not divide `ptr`)
    ///
    /// # Safety
    /// - The given type must have correct lifespan as it was pushed into the stack
    pub unsafe fn access<E>(&self, ptr: usize) -> Option<&E>
    where
        E: Sized,
    {
        let size: usize = core::mem::size_of::<E>();
        let ptr = self.mem.access(ptr, size)?;
        Some(unsafe { &*(ptr.as_ptr() as *const E) })
    }

    /// Returns the size of used bytes of stack. This is lower than the actual memory cost,
    /// as there may be some vacant pending memory to be used.
    pub fn size(&self) -> usize {
        self.top
    }
}
