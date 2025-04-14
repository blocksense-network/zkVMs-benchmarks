#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(feature = "sp1")]
use sha2_sp1::{Digest, Sha256};

#[cfg(feature = "risc0")]
use sha2_risc0::{Digest, Sha256};

#[cfg(not(any(feature = "sp1", feature = "risc0")))]
use sha2::{Digest, Sha256};

#[guests_macro::proving_entrypoint]
pub fn main(secret: Vec<u8>, hash: Vec<u8>) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    let result = hasher.finalize();

    let output: [u8; 32] = result.into();

    output.to_vec() == hash
}
