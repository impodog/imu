mod mem;
pub use mem::Memory;

#[cfg(feature = "vec_mem")]
mod vec_mem;
#[cfg(feature = "vec_mem")]
pub use vec_mem::VecMem;

/// Returns whether the pointer is properly aligned on the target.
/// I.e. The pointer is divisible by `crate::ALIGNMENT`
pub fn is_aligned_target<T>(ptr: *const T) -> bool {
    ptr as usize % crate::ALIGNMENT == 0
}
