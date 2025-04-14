#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

#[cfg(feature = "sp1")]
use sha3_sp1::{Digest, Keccak256};

#[cfg(feature = "risc0")]
use sha3_risc0::{Digest, Keccak256};

#[cfg(not(any(feature = "zkm", feature = "sp1", feature = "risc0")))]
use sha3::{Digest, Keccak256};

#[guests_macro::proving_entrypoint]
pub fn main(secret: Vec<u8>, hash: Vec<u8>) -> bool {
    #[cfg(feature = "zkm")]
    let result = zkm_runtime::io::keccak(&secret.as_slice());

    #[cfg(not(feature = "zkm"))]
    let result = {
        let mut hasher = Keccak256::new();
        hasher.update(secret);
        hasher.finalize()
    };

    let output: [u8; 32] = result.into();

    output.to_vec() == hash
}
