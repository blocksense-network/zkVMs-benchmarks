#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use sha3::{Digest, Keccak256};

#[guests_macro::proving_entrypoint]
pub fn main(secret: Vec<u8>, hash: Vec<u8>) -> bool {
    let mut hasher = Keccak256::new();
    hasher.update(secret);
    let result = hasher.finalize();

    let output: [u8; 32] = result.into();

    output.to_vec() == hash
}
