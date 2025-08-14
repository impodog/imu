#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;
use spin::{RwLock, RwLockReadGuard, RwLockWriteGuard};

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::ptr;
    } else {
        use core::ptr;
    }
}

use ptr::NonNull;

use imur_mem::alloc::Memory;
use imur_mem::heap::HeapAlloc;

pub mod proc;
pub mod th;

use proc::Proc;
use th::Th;
