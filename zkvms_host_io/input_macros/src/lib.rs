use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::FunctionDefinition;

/// Parses the `guests/type.txt` type note, created from the guest
/// Returns a tuple of the arguments group and the return type
fn new_fd() -> FunctionDefinition {
    FunctionDefinition::new(&include_str!("../../../guests/type.txt").parse().unwrap())
}

static DERIVES: &str = "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]";

/// Creates an Output type def and three Input structures from the guest
/// type.txt file.
///
/// # Usage
///
/// Inside zkvms_host_io:
///
/// ```rust
/// input_macros::generate_output_type_input_struct!();
/// ```
///
/// # Example output
///
/// ```rust
/// pub type Output = (... ...);
///
/// pub type Return = ...;
///
/// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// pub struct PublicInput {
///     pub ...: ...,
///     pub ...: ...,
///     ...
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// pub struct PrivateInput {
///     pub ...: ...,
///     pub ...: ...,
///     ...
/// }
///
/// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
/// pub struct Input {
///     pub ...: ...,
///     pub ...: ...,
///     ...
/// }
///
/// // Converts Input to a tuple
/// impl From<Input> for (...) {
///     fn from(input: Input) -> (...) {
///         (
///             input....,
///             input....,
///             ...
///         )
///     }
/// }
/// ```
#[proc_macro]
pub fn generate_output_type_input_struct(_: TokenStream) -> TokenStream {
    let fd = new_fd();

    let sep = if fd.public_types().is_empty() { "" } else { ", " };
    let output_type = format!("pub type Output = ({} {} {});", fd.grouped_public_types(), sep, fd.return_type).to_string();

    let return_type = format!("pub type Return = {};", fd.return_type).to_string();

    let public_attrs = fd
        .public_arguments()
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let public_input_type =
        format!("{} pub struct PublicInput {{ {} }}", DERIVES, public_attrs).to_string();

    let private_attrs = fd
        .private_arguments()
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let private_input_type = format!(
        "{} pub struct PrivateInput {{ {} }}",
        DERIVES, private_attrs
    )
    .to_string();

    let attrs = fd
        .arguments()
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let convertion = fd
        .patterns()
        .clone()
        .iter()
        .map(|x| format!("input.{x},"))
        .collect::<String>();
    let types = fd.grouped_types();
    let struct_def = &format!("
        {DERIVES} pub struct Input {{
            {attrs}
        }}
        impl From<Input> for ({types}) {{
            fn from(input: Input) -> ({types}) {{
                ({convertion})
            }}
        }}
    ").to_string();

    (output_type + &return_type + &public_input_type + &private_input_type + &struct_def)
        .parse::<TokenStream>()
        .unwrap()
}

/// Repeats the given item as many times as fields there are, while replacing
/// all `.yield` occurences with the fields value (field name).
fn foreach_field(item: TokenStream, fields: Vec<TokenStream>) -> TokenStream {
    let expr = format!("{}", item);
    let mut out = String::new();
    for field in fields {
        // Unquoted yield is a keyword, so it is not allowed as field name
        out += &expr.replace(".yield", &format!(".{field}"));
    }
    out.parse::<TokenStream>().unwrap()
}

/// Repeats the given code as many times as fields there are in the Input
/// struct, while replacing all `.yield` occurences with the concrete
/// field name.
#[proc_macro]
pub fn foreach_input_field(item: TokenStream) -> TokenStream {
    foreach_field(item, new_fd().patterns().clone())
}

/// Repeats the given code as many times as fields there are in the
/// PublicInput struct, while replacing all `.yield` occurences with the
/// concrete field name.
#[proc_macro]
pub fn foreach_public_input_field(item: TokenStream) -> TokenStream {
    foreach_field(item, new_fd().public_patterns().clone())
}

/// Repeats the given code as many times as fields there are in the
/// PrivateInput struct, while replacing all `.yield` occurences with the
/// concrete field name.
#[proc_macro]
pub fn foreach_private_input_field(item: TokenStream) -> TokenStream {
    foreach_field(item, new_fd().private_patterns().clone())
}

/// Assuming the `run_info` variable is present, it creates a block with all
/// needed code to properly benchmark the input code, according to all command
/// parameters.
#[proc_macro]
pub fn benchmarkable(item: TokenStream) -> TokenStream {
    format!(r#"
        {{
             use std::time::Instant;

             let mut starts = Vec::new();
             let mut ends = Vec::new();

             for i in 1..=run_info.repeats {{
                 if run_info.benchmarking {{
                     starts.push(Instant::now());
                 }}

                 {item}

                 if run_info.benchmarking {{
                     ends.push(Instant::now());
                 }}
             }}

             if run_info.benchmarking {{
                 zkvms_host_io::emit_benchmark_results(run_info, starts, ends);
             }}
        }}
    "#).parse().unwrap()
}
