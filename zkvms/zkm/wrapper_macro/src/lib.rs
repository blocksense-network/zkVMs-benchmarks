use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{
    args_divide_grouped, args_divide_public, args_split, group_streams, split_fn,
};

/// Create a body, which reads all public and private inputs, stores them in
/// variables, then executes the guest entrypoint function with those arguments
/// and commits its output.
///
/// `default_public_input.toml` shows which variables are public.
///
/// # Usage
///
/// Inside ZKM's guest (excluding the `entrypoint_expr` call):
///
/// ```rust
/// make_wrapper!{fn main(...) -> ...}
/// ```
///
/// # Example output
///
/// ```rust
/// {
///     let (...) : (...) = read(); // Public inputs
///     let (...) : (...) = read(); // Private inputs
///     commit::<...>(&zkp::main(..., ..., ...));
/// }
/// ```
#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let (ts_patterns, _) = args_divide_grouped(&args);

    let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
        env!("INPUTS_DIR"),
        "/default_public_input.toml"
    )))
    .unwrap();
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) =
        args_divide_public(&args, &public_inputs.keys().collect());
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = (
        (group_streams(&pub_pat), group_streams(&pub_typ)),
        (group_streams(&prv_pat), group_streams(&prv_typ)),
    );

    let mut out = TokenStream::new();
    // NOTE: The first read returns public data, the second returns private
    out.extend(format!("let {} : {} = read();", pub_pat, pub_typ).parse::<TokenStream>());
    out.extend(format!("let {} : {} = read();", prv_pat, prv_typ).parse::<TokenStream>());

    out.extend(format!("commit::<{}>(&zkp::{}{});", ret, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
