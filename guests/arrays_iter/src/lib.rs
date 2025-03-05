#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[guests_macro::proving_entrypoint]
pub fn main(numbers: Vec<i32>, remainder: i32, divisor: i32) -> bool {
    for n in numbers {
        if n % divisor != remainder {
            return false;
        }
    }
    return true;
}
