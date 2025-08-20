mod types;
pub use types::HeapAlloc;

#[cfg(feature = "bare_heap")]
pub mod bare_heap;

#[cfg(feature = "sync_heap")]
pub mod sync_heap;

#[cfg(feature = "locked_heap")]
pub mod locked_heap;
