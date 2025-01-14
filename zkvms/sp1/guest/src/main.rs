#![no_main]

use wrapper_macro::make_wrapper;
use sp1_zkvm::io::read;
use sp1_zkvm::lib::io::commit;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    zkp::entrypoint_expr!()
}
