use crate::alloc::Memory;
use crate::*;

/// An IMU stack that manages its internal memory, supporting typical stack operations.
/// Please note that the stack does not support variability after the element is inserted.
pub struct Stack<M: Memory> {
    mem: M,
    top: usize,
}

impl<M: Memory> Stack<M> {
    /// Creates an empty stack holding a memory
    pub fn new(mem: M) -> Self {
        Self { mem, top: 0 }
    }

    /// Extends the size of the stack to accommodate new bytes,
    /// which may or may not trigger an allocation.
    /// This does not change the top of the stack, but may change the memory.
    /// Returns whether allocation(if any) is successful.
    #[must_use]
    fn extend_by(&mut self, size: usize) -> bool {
        let rem = self.mem.size() - self.top;
        if rem < size {
            let extra = size - rem;
            self.mem.grow_more(extra)
        } else {
            true
        }
    }

    /// Pushes an element of given size into the stack, returning its offset,
    /// which can be used to access that element later.
    /// If allocation fails by the internal `Memory`, `None` is returned.
    pub fn push<E>(&mut self, elem: E) -> Option<usize>
    where
        E: Sized,
    {
        let size: usize = mem::size_of::<E>();
        if !self.extend_by(size) {
            return None;
        }

        let vptr = self.top;
        // Stores the value
        let ptr = self
            .mem
            .access_mut(vptr, size)
            .expect("Should contain the stack element");
        unsafe {
            ptr::write_unaligned(ptr.as_ptr() as *mut E, elem);
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

    /// Accesses the stack for the element at pointer, or return `None` if the pointer is out of
    /// bounds, or the size of the element would exceed to size limits.
    ///
    /// # Panics
    /// - Panics when the pointer is misaligned(i.e. `crate::ALIGNMENT` does not divide `ptr`)
    ///
    /// # Safety
    /// - The given type must be the same as the value was pushed into the stack
    pub unsafe fn access_aligned<E>(&self, ptr: usize) -> Option<&E>
    where
        E: Sized,
    {
        let size: usize = mem::size_of::<E>();
        let ptr = self.mem.access(ptr, size)?;
        Some(unsafe { &*(ptr.as_ptr() as *const E) })
    }

    /// Accesses the stack for the element at pointer, or return `None` if the pointer is out of
    /// bounds, or the size of the element would exceed to size limits.
    ///
    /// # Safety
    /// - The given type must be the same as the value was pushed into the stack
    pub unsafe fn access<E>(&self, ptr: usize) -> Option<E>
    where
        E: Sized + Copy,
    {
        let size: usize = mem::size_of::<E>();
        let ptr = self.mem.access(ptr, size)?;
        Some(unsafe { (ptr.as_ptr() as *const E).read_unaligned() })
    }

    /// Duplicates the memory that is already stored in the stack to the top of the stack.
    /// This function ensures safe memory behavior.
    ///
    /// Returns the offset to the newly-created element, if the operation is successful. This fails because of internal memory
    /// allocation failure, or if `ptr` of `size` is out of bounds
    #[must_use]
    pub fn duplicate(&mut self, ptr: usize, size: usize) -> Option<usize> {
        if ptr + size > self.top {
            return None;
        }
        if !self.extend_by(size) {
            return None;
        }
        let src = self
            .mem
            .access_mut(ptr, size)
            .expect("ptr, size are checked to be inside the stack");
        let dst = self
            .mem
            .access_mut(self.top, size)
            .expect("new allocation is made sure to accommodate size bytes");
        unsafe {
            // NOTE: `ptr` is ensured to be inside the old stack, thus there is no memory overlap.
            ptr::copy_nonoverlapping(src.as_ptr() as *const u8, dst.as_ptr() as *mut u8, size);
        }
        let result = self.top;
        self.top += size;
        Some(result)
    }

    /// Returns the size of used bytes of stack. This is lower than or equal to the actual memory cost,
    /// as there may be some vacant pending memory to be used.
    pub fn size(&self) -> usize {
        self.top
    }
}
