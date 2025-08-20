use crate::*;

/// All information required to run an IMU emulated process.
/// Holds its threads, imports, and other configs.
///
/// This relies on one heap, which requires user-provided linear memory.
pub struct Proc<S: Memory, H: HeapAlloc + Sync, T: SysTh> {
    th: Vec<Th<S, H, T>, H>,
    sys: Vec<T, H>,
    wait: Vec<Deque<usize, H>, H>,
}

impl<S: Memory, H: HeapAlloc + Sync + Clone, T: SysTh> Proc<S, H, T> {
    /// Creates a new process running on current thread.
    /// You must provide at least one system thread adapter.
    ///
    /// This only returns `None` when internal heap allocation fails.
    ///
    /// # Panic
    ///
    /// Panics if the given `sys` is empty.
    pub fn new(heap: H, sys: impl core::iter::IntoIterator<Item = T>) -> Option<Self> {
        let mut this = Self {
            th: Vec::new(heap.clone()),
            sys: Vec::new(heap.clone()),
            wait: Vec::new(heap.clone()),
        };
        for sys in sys.into_iter() {
            if !this.sys.push(sys) {
                return None;
            }
        }
        if this.sys.is_empty() {
            panic!("no system thread given");
        }
        Some(this)
    }
}
