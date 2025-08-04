use crate::*;

/// Defines an appropriate interface for IMU stack and heap allocations. The internal allocation is
/// handled by the user type, while the the stack and heap management is done by IMUR.
/// This interface should be applicable for all architectures and very simple to implement.
pub trait Memory {
    /// Requests for more usable memory.
    ///
    /// The backend can define the strategy to grow bytes,
    /// but have to guarantee a non-negative change in usable memory.
    /// The backend does not need to guarantee any initial values in new bytes.
    fn grow(&mut self);

    /// Requests for more memory with a minimum bound, returning if successful.
    ///
    /// The backend can define the strategy to grow bytes,
    /// but if returning true have to guarantee the number of increased bytes is
    /// not lower than `min`, or if returning false, the number of usable bytes must not change.
    /// The backend does not need to guarantee any initial values in new bytes.
    fn grow_more(&mut self, min: usize) -> bool;

    /// Accesses the rust-level memory pointer positioned in the memory, or return [`None`] if any parts of the
    /// requested memory is out of bounds
    ///
    /// # Safety
    /// - Calling any grow functions can *invalidate* the pointer returned.
    /// - The pointer if returned is valid to *read* within given bounds, but *not write*.
    ///   If you need to write to the memory, use [`Memory::access_mut`].
    /// - The type of return pointer is unspecified and not guaranteed to be valid for every type.
    ///   However numeric types of the given size are always valid as they do not require alignment.
    fn access(&self, ptr: usize, size: usize) -> Option<NonNull<()>>;

    /// This is the mutually exclusive version of [`Memory::access`]. See it for details.
    fn access_mut(&mut self, ptr: usize, size: usize) -> Option<NonNull<()>>;

    /// Returns the exact size of bytes usable by the external program.
    fn size(&self) -> usize;
}
