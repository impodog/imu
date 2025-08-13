#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::vec::Vec;
        use std::ptr::NonNull;
        use std::{ptr, mem, ops};
    } else {
        use core::ptr::NonNull;
        use core::{ptr, mem, ops};
    }
}
const ALIGNMENT: usize = mem::align_of::<*const ()>();

pub mod alloc;
pub mod heap;
pub mod stack;
