use crate::*;

/// All information required to run an IMU emulated process.
/// Holds its threads, imports, and other configs.
///
/// This relies on one heap, which requires user-provided linear memory.
pub struct Proc<S: Memory, H: Memory, T: SysTh> {
    th: Vec<Th<S, H, T>, SyncHeap<H>>,
    sys: Vec<T, SyncHeap<H>>,
}

impl<S: Memory, H: Memory, T: SysTh> Proc<S, H, T> {}
