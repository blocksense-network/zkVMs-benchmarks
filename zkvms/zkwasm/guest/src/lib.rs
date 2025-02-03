use wasm_bindgen::prelude::wasm_bindgen;
use wrapper_macro::make_wrapper;
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

static VERTICES: u64 = 10;

macro_rules! read {
    // HACK for graph_coloring
    (Vec , u32 $readfn:tt) => {
        {
            let mut ret = Vec::new();
            for _ in 0..2 {
                ret.push(read!(u32 $readfn));
            }
            ret
        }
    };
    // Vec<Vec<...<Vec<primitive>>>> is converted by entrypoint_expr! to
    // Vec,Vec,...,Vec,primitive
    (Vec , $($type:tt)*) => {
        {
            let mut ret = Vec::new();
            for _ in 0..VERTICES {
                ret.push(read!($($type)*));
            }
            ret
        }
    };
    (bool $readfn:tt) => {
        ($readfn() != 0)
    };
    // Has to be primitive!
    ($type:tt $readfn:tt) => {
        ($readfn() as $type)
    };
}

#[wasm_bindgen]
pub fn zkmain() {
    zkp::entrypoint_expr!()
}
