#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
    } else {
    }
}

pub mod ctx;
