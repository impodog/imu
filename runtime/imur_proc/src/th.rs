use crate::*;

/// An IMU emulated thread under `crate::proc::Proc` with specific execution info.
pub struct Th<S: Memory, H: Memory> {
    /// Program command index, which is unique globally.
    pub pc: usize,
    /// Pointer back to parent process, must be always available, guaranteed
    /// by freeing threads first.
    proc: NonNull<Proc<S, H>>,
}
