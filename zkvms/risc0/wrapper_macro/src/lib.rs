use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::FunctionDefinition;

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
    let fd = FunctionDefinition::new(&item);

    let args = fd
        .arguments()
        .into_iter()
        .map(|x| format!("let {x} = read();"))
        .collect::<String>();

    let mut out = TokenStream::new();
    out.extend(args.parse::<TokenStream>());

    let commits = fd
        .public_patterns()
        .clone()
        .into_iter()
        .map(|x| format!("commit(&{x});"))
        .collect::<String>();
    out.extend(commits.parse::<TokenStream>());

    out.extend(
        format!("commit(&zkp::{}({}));", fd.name, fd.grouped_patterns()).parse::<TokenStream>(),
    );

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
