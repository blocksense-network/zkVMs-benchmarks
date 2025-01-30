#![cfg_attr(feature = "no_std", no_std)]

#[guests_macro::proving_entrypoint]
pub fn main(n: u8, fN: u64) -> bool {
    let mut f0 = 0;
    let mut f1 = 1;

    for _ in 0..n {
        let fN = f0 + f1;
        f0 = f1;
        f1 = fN;
    }

    f0 == fN
}
