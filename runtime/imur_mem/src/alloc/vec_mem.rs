use crate::alloc::Memory;
use crate::*;

/// An utility memory allocator by `std::vec::Vec`. Only available with feature "std".
/// This grows by multiplier of 2 each time `Self::grow` is called if the vec is above half full.
/// The reserving behavior is the same as how `std::vec::Vec` is implemented in `Vec::resize`.
#[derive(Default)]
pub struct VecMem {
    mem: Vec<u8>,
}

impl VecMem {
    /// Creates a new memory containing an empty `Vec`
    pub fn new() -> Self {
        Self::default()
    }
}

impl Memory for VecMem {
    fn grow(&mut self) {
        if self.mem.is_empty() {
            self.mem.push(0);
        } else {
            let len = self.mem.len().saturating_mul(2);
            if len > self.mem.capacity() {
                self.mem.resize(len, 0);
            }
        }
    }

    fn grow_more(&mut self, incr: usize) -> bool {
        let new_len = self.mem.len().saturating_add(incr);
        if new_len > isize::MAX as usize {
            false
        } else {
            self.mem.resize(new_len, 0);
            true
        }
    }

    fn shrink(&mut self, min: usize) {
        let dbl = min.saturating_mul(3);
        if dbl <= self.mem.len() {
            self.mem.truncate(min);
        }
    }

    fn access(&self, ptr: usize, size: usize) -> Option<NonNull<()>> {
        if ptr + size > self.mem.len() {
            None
        } else {
            let ptr = unsafe { self.mem.as_ptr().add(ptr) };
            self.mem.as_ptr();
            Some(unsafe { NonNull::new_unchecked(ptr as *mut ()) })
        }
    }

    fn access_mut(&mut self, ptr: usize, size: usize) -> Option<NonNull<()>> {
        self.access(ptr, size)
    }

    fn size(&self) -> usize {
        self.mem.len()
    }
}
