#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::ptr::NonNull;
        use std::{ptr, mem};
        use std::clone::Clone;
        use std::marker::Copy;
    } else {
        use core::ptr::NonNull;
        use core::{ptr, mem};
        use core::clone::Clone;
        use core::marker::Copy;
    }
}
cfg_if! {
    if #[cfg(feature = "vec_mem")] {
        use std::vec::Vec;
    }
}
const ALIGNMENT: usize = mem::align_of::<*const ()>();

pub mod alloc;
pub mod ds;
pub mod heap;
pub mod stack;
