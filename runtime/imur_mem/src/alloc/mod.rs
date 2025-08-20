mod mem;
pub use mem::Memory;

#[cfg(feature = "vec_mem")]
pub mod vec_mem;

#[cfg(feature = "arc")]
pub mod arc;

/// Returns whether the pointer is properly aligned on the target.
/// I.e. The pointer is divisible by `crate::ALIGNMENT`
pub fn is_aligned_target<T>(ptr: *const T) -> bool {
    ptr as usize % crate::ALIGNMENT == 0
}
