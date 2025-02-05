use wasm_bindgen::prelude::wasm_bindgen;
use wrapper_macro::{ make_wrapper, read };
// https://github.com/DelphinusLab/zkWasm-rust/blob/main/src/lib.rs
use zkwasm_rust_sdk::{require, wasm_input, wasm_output};

fn read_private() -> u64 {
    unsafe { wasm_input(0) }
}

fn read_public() -> u64 {
    unsafe { wasm_input(1) }
}

fn assert(cond: bool) {
    unsafe { require(cond); }
}

fn write(value: u64) {
    unsafe { wasm_output(value); }
}

#[wasm_bindgen]
pub fn zkmain() {
    zkp::entrypoint_expr!()
}
