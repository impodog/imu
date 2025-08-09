use imuc_ir::cmd::Ptr;
use std::sync::LazyLock;

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
        align_ptr_to(ptr, self.ptr_align)
    }

    /// Returns the initial alignment size, respecting `Self::loose_align` and `Self::ptr_align`
    pub fn init_align(&self) -> usize {
        if self.loose_align {
            self.ptr_align
        } else {
            0
        }
    }

    /// When iterating through fields, update the alignment using the current field size,
    /// useful for creating padding. Alignment is capped at `Self::ptr_align`
    pub fn update_align_by(&self, align: usize, field_size: imuc_ir::cmd::Bytes) -> usize {
        if field_size.num() > align {
            let align = match field_size.num() {
                0..2 => 1,
                2 => 2,
                3..5 => 4,
                5..9 => 8,
                _ => 16,
            };
            align.max(self.ptr_align)
        } else {
            align
        }
    }
}

/// Aligns the ptr to the given alignmen. If the ptr is already aligned, it is simply returned.
/// Does nothing if align == 0 or 1
pub fn align_ptr_to(ptr: Ptr, align: usize) -> Ptr {
    if align == 0 || align == 1 {
        ptr
    } else {
        let rem = ptr.num() % align;
        if rem == 0 {
            ptr
        } else {
            Ptr::new(ptr.num() + align - rem)
        }
    }
}

pub static MEMORY_LAYOUT: LazyLock<MemoryLayout> = LazyLock::new(Default::default);
