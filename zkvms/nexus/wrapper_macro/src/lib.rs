use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide_public, args_divide_grouped };

/// Creates a body, which reads all inputs, stores them in variables, then
/// writes the ones, defined as public in `default_public_input.toml` to the
/// journal and finally executes the guest entrypoint function with those
/// arguments, committing its output.
///
/// # Usage
///
/// Inside Nexus' guest (excluding the `entrypoint_expr` call):
///
/// ```rust
/// make_wrapper!{fn main(...) -> ...}
/// ```
///
/// # Example output
///
/// ```rust
/// {
///     let ... = read_private_input::<...>().unwrap();
///     let ... = read_private_input::<...>().unwrap();
///     ...
///     write_output::<...>(&...);
///     write_output::<...>(&...);
///     ...
///     write_output::<...>(&zkp::main(..., ..., ...));
/// }
/// ```
#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let (ts_patterns, ts_types) = args_divide_grouped(&args);

    let mut out = TokenStream::new();
    out.extend(format!("let {} = read_private_input::<{}>().unwrap();", ts_patterns, ts_types).parse::<TokenStream>());

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let (public_patterns, public_types) = args_divide_public(&args, &public_inputs.keys().collect()).0;
    let public_patterns: Vec<(TokenStream, TokenStream)> = public_patterns
        .into_iter()
        .zip(public_types.into_iter())
        .collect();
    for (pattern, ptype) in public_patterns {
        out.extend(format!("write_output::<{}>(&{});", ptype, pattern).parse::<TokenStream>());
    }

    out.extend(format!("write_output::<{}>(&zkp::{}{});", ret, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
