use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::FunctionDefinition;

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
    let fd = FunctionDefinition::new(&item);

    let mut out = TokenStream::new();
    // NOTE: The first read returns public data, the second returns private
    out.extend(
        format!(
            "let ({}) : ({}) = read();",
            fd.grouped_public_patterns(),
            fd.grouped_public_types()
        )
        .parse::<TokenStream>(),
    );
    out.extend(
        format!(
            "let ({}) : ({}) = read();",
            fd.grouped_private_patterns(),
            fd.grouped_private_types()
        )
        .parse::<TokenStream>(),
    );

    out.extend(
        format!(
            "commit::<{}>(&zkp::{}({}));",
            fd.return_type,
            fd.name,
            fd.grouped_patterns()
        )
        .parse::<TokenStream>(),
    );

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
