#![cfg_attr(feature = "no_std", no_std)]

#[cfg(feature = "no_std")]
extern crate alloc;
#[cfg(feature = "no_std")]
use alloc::vec::Vec;

use sha2::{Digest, Sha256};

#[cfg(feature = "sp1")]
use rsa_sp1::{pkcs8::DecodePublicKey, Pkcs1v15Sign, RsaPublicKey};

#[cfg(feature = "risc0")]
use rsa_risc0::{pkcs8::DecodePublicKey, Pkcs1v15Sign, RsaPublicKey};

#[cfg(not(any(feature = "sp1", feature = "risc0")))]
use rsa::{pkcs8::DecodePublicKey, Pkcs1v15Sign, RsaPublicKey};

#[guests_macro::proving_entrypoint]
pub fn main(public_key: Vec<u8>, message: String, signature: Vec<u8>) -> bool {
    let public_key = RsaPublicKey::from_public_key_der(&public_key).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(message);
    let hashed_msg = hasher.finalize();

    public_key
        .verify(Pkcs1v15Sign::new::<Sha256>(), &hashed_msg, &signature)
        .is_ok()
}
