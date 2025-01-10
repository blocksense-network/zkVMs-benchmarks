use wrapper_macro::make_wrapper;
use risc0_zkvm::guest::env::{ read, commit };

fn main() {
    zkp::entrypoint_expr!()
}
