#![cfg_attr(feature = "no_std", no_std)]

use hex_literal::hex;

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use sha2::{Sha256, Digest};

#[guests_macro::proving_entrypoint]
pub fn main(public_digest: String, private_message: String) -> usize {
    let mut hasher = Sha256::new();
    hasher.update(private_message.as_bytes());

    let digest = hasher.finalize();
    assert_eq!(digest[..], hex!(public_digest)[..]);

    5
}
