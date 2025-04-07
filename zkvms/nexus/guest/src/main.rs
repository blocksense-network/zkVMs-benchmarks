#![cfg_attr(target_arch = "riscv32", no_std, no_main, allow(unused_imports))]

use nexus_rt::{postcard, println, read_private_input, read_public_input, write_public_output};

extern crate alloc;
use alloc::{collections::*, vec::*};
use wrapper_macro::make_wrapper;

#[nexus_rt::main]
fn main() {
    zkp::entrypoint_expr!()
}
