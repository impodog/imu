mod mem;
pub use mem::Memory;

#[cfg(feature = "std")]
mod std_mem;
#[cfg(feature = "std")]
pub use std_mem::VecMem;
