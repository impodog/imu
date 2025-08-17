#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;
use spin::RwLock;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::ptr;
        use std::clone::Clone;
        use std::marker::Copy;
    } else {
        use core::ptr;
        use core::clone::Clone;
        use core::marker::Copy;
    }
}

use ptr::NonNull;

use imur_mem::alloc::Memory;
use imur_mem::ds::vec::Vec;
use imur_mem::heap::HeapAlloc;
use imur_mem::heap::sync_heap::SyncHeap;

pub mod proc;
pub mod th;
pub mod types;

pub use types::SysTh;

use proc::Proc;
use th::Th;
