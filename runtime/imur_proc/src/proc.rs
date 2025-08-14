use crate::*;

/// All information required to run an IMU emulated process.
/// Holds its threads, imports, and other configs.
///
/// This relies on one stack and one heap, which requires user-provided linear memory,
/// and IMU adapters for them. `S` is the type of stack memory, `H` is the type of heap memory.
pub struct Proc<S: Memory, H: Memory> {
    stack: imur_mem::stack::Stack<S>,
    heap: imur_mem::heap::Heap<H>,
}
