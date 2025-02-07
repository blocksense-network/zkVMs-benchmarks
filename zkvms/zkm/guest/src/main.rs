#![no_std]
#![no_main]

use wrapper_macro::make_wrapper;

extern crate alloc;
use alloc::{ vec::*, collections::* };
use zkm_runtime::io::{ read, commit };

zkm_runtime::entrypoint!(main);

pub fn main() {
    zkp::entrypoint_expr!()
}
