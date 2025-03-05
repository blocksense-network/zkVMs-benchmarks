#![cfg_attr(feature = "no_std", no_std)]

use nalgebra::Matrix2;

#[guests_macro::proving_entrypoint]
pub fn main(n: u8, fN: u64) -> bool {
    let r = Matrix2::new(1, 1, 1, 0).pow((n - 1).into())[(0, 0)];

    r == fN
}
