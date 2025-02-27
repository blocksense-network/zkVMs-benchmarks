use proc_macro::TokenStream;
use std::{fs::File, io::Write};
mod parse_fn;

/// Create an `entrypoint_expr` macro inside the guest program. This will be
/// used inside the ZKVM guest program to call the ZKVM guest wrapper's
/// `make_wrapper` macro with entrypoint function type.
///
/// The overarching goal is to call `make_wrapper` with an input, which
/// contains the function definition. This can only happen inside the guest,
/// since that is the only place where we have the definition (syntactically).
/// But you can only pass data to macros by "inlining" it as macro arguments
/// (i.e. macros work on syntax, so creating a variable wouldn't work). Also,
/// the `make_wrapper` definition doesn't exist in the guest, but the ZKVM
/// wrapper crate. For these reasons we create a macro which invokes
/// `make_wrapper` with the proper arguments.
///
/// # Usage
///
/// Inside your guest (under guests directory) add an attribute above your main
/// (entrypoint/start) function. It takes no arguments.
///
/// ```rust
/// #[guests_macro::proving_entrypoint]
/// fn main(...) -> ... { ..... }
/// ```
///
/// # Example output
///
/// ```rust
/// #[macro_export]
/// macro_rules! entrypoint_expr {
///     () => {
///         make_wrapper!{fn main(...) -> ...}
///     };
/// }
/// ```
#[proc_macro_attribute]
pub fn proving_entrypoint(_: TokenStream, mut item: TokenStream) -> TokenStream {
    let (name, args, ret) = parse_fn::split_fn(&item);

    // We also need to pass some type information to the host program compile-time.
    // Put it in the file guests/type.txt.
    let mut output = File::create("../type.txt").unwrap();
    writeln!(output, "{}", &format!("{args}").replace('\n', " "));
    write!(output, "{}", &format!("{ret}").replace('\n', " "));

    item.extend(
        format!(
            "#[macro_export]
        macro_rules! entrypoint_expr {{
            () => {{
                make_wrapper!{{{}{} -> {}}}
            }};
        }}",
            name, args, ret
        )
        .parse::<TokenStream>(),
    );
    item
}
