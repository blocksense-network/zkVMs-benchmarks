#![no_std]
#![no_main]

use wrapper_macro::make_wrapper;

extern crate alloc;
use alloc::{collections::*, vec::*, string::*};
use zkm_runtime::io::{commit, read};

zkm_runtime::entrypoint!(main);

pub fn main() {
    zkp::entrypoint_expr!()
}
