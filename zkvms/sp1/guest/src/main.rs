#![no_main]

use sp1_zkvm::io::read;
use sp1_zkvm::lib::io::commit;
use std::collections::*;
use wrapper_macro::make_wrapper;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    zkp::entrypoint_expr!()
}
