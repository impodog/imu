#![no_std]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::vec::Vec;
        use std::ptr::NonNull;
    } else {
        use core::ptr::NonNull;
    }
}

pub mod alc;
