#![cfg_attr(not(feature = "std"), no_std)]

use cfg_if::cfg_if;
use critical_section::Mutex;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
    } else {
    }
}
