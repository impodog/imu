use std::sync::LazyLock;

/// Defines the compiler's behavior on memory layout
pub struct MemoryLayout {
    /// The output code alignment on stack and tuple/cus fields
    pub ptr_align: usize,
    /// Aligns every single value to `Self::ptr_align`, even in cus fields
    pub loose_align: bool,
    /// Prevents the compiler from sorting tuple/cus fields and keep original order
    pub no_sort: bool,
}

impl Default for MemoryLayout {
    fn default() -> Self {
        Self {
            ptr_align: std::mem::align_of::<*const ()>(),
            loose_align: false,
            no_sort: false,
        }
    }
}

pub static MEMORY_LAYOUT: LazyLock<MemoryLayout> = LazyLock::new(Default::default);
