use std::sync::LazyLock;

use imuc_ir::cmd::Ptr;

/// Defines the compiler's behavior on memory layout
pub struct MemoryLayout {
    /// The output code alignment on stack and tuple/cus fields
    pub ptr_align: usize,
    /// Aligns every single value to `Self::ptr_align`, even in cus fields
    pub loose_align: bool,
}

impl Default for MemoryLayout {
    fn default() -> Self {
        Self {
            ptr_align: std::mem::align_of::<*const ()>(),
            loose_align: false,
        }
    }
}

impl MemoryLayout {
    /// Aligns the ptr *up* to match alignment settings. If the ptr is already aligned, it is
    /// simply returned
    pub fn align_ptr(&self, ptr: Ptr) -> Ptr {
        let rem = ptr.num() % self.ptr_align;
        if rem == 0 {
            ptr
        } else {
            Ptr::new(ptr.num() + self.ptr_align - rem)
        }
    }
}

pub static MEMORY_LAYOUT: LazyLock<MemoryLayout> = LazyLock::new(Default::default);
