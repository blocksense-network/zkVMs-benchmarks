use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{args_divide_grouped, args_divide_public, args_split, split_fn};

/// Create a body, which reads all inputs, stores them in variables, then
/// commits the ones, defined as public in `default_public_input.toml` to the
/// journal and finally executes the guest entrypoint function with those
/// arguments, committing its output.
///
/// # Usage
///
/// Inside RISC0's guest (excluding the `entrypoint_expr` call):
///
/// ```rust
/// make_wrapper!{fn main(...) -> ...}
/// ```
///
/// # Example output
///
/// ```rust
/// {
///     let ... : ... = read();
///     let ... : ... = read();
///     ...
///     commit(&...);
///     commit(&...);
///     ...
///     commit(&zkp::main(..., ..., ...));
/// }
/// ```
#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);
    let args_split = args_split(&args);

    let mut out = TokenStream::new();
    for arg in args_split {
        out.extend(format!("let {} = read();", arg).parse::<TokenStream>());
    }

    let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
        env!("INPUTS_DIR"),
        "/default_public_input.toml"
    )))
    .unwrap();
    let public_patterns = args_divide_public(&args, &public_inputs.keys().collect())
        .0
         .0;
    for pattern in public_patterns.iter() {
        out.extend(format!("commit(&{});", pattern).parse::<TokenStream>());
    }

    let (ts_patterns, _) = args_divide_grouped(&args);

    out.extend(format!("commit(&zkp::{}{});", name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
