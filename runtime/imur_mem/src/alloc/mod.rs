mod mem;
pub use mem::Memory;

#[cfg(feature = "std")]
mod std_mem;
#[cfg(feature = "std")]
pub use std_mem::VecMem;

/// Returns whether the pointer is properly aligned on the target.
/// I.e. The pointer is divisible by `crate::ALIGNMENT`
pub fn is_aligned_target<T>(ptr: *const T) -> bool {
    ptr as usize % crate::ALIGNMENT == 0
}
