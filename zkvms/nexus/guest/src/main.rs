#![cfg_attr(target_arch = "riscv32", no_std, no_main, allow(unused_imports))]

use nexus_rt::{ postcard, println, read_private_input, write_output };

extern crate alloc;
use alloc::vec::Vec;
use wrapper_macro::make_wrapper;

type Input = (Vec<Vec<bool>>, u32, Vec<Vec<u32>>);
type Output = bool;

const VERTICES: usize = 100;

#[nexus_rt::main]
fn main() {
    zkp::entrypoint_expr!()
}
