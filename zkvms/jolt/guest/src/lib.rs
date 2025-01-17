//#![cfg_attr(feature = "guest", no_std)]
#![no_main]

use wrapper_macro::make_wrapper;

zkp::entrypoint_expr!{}
