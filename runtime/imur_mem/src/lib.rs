#![no_std]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::vec::Vec;
        use std::ptr::NonNull;
        const ALIGNMENT: usize = std::mem::align_of::<*const ()>();
    } else {
        use core::ptr::NonNull;
        const ALIGNMENT: usize = core::mem::align_of::<*const ()>();
    }
}

pub mod alloc;
pub mod heap;
pub mod stack;
