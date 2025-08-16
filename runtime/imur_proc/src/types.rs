/// Defines an appropriate interface for IMU thread emulation.
/// You may simply execute the function if your program runs on a single thread.
pub trait SysTh: Sized {
    /// Executes the function on the thread. On a thread different than main thread,
    /// this function should not block.
    fn execute<F>(f: F)
    where
        F: FnOnce();
}
