use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{
    args_divide, args_divide_public, args_split, args_split_public, group_streams,
};

/// Parses the `guests/type.txt` type note, created from the guest
/// Returns a tuple of the arguments group and the return type
fn get_types() -> (TokenStream, TokenStream) {
    let types: Vec<&str> = include_str!("../../../guests/type.txt")
        .split('\n')
        .collect();
    (
        types[0].parse::<TokenStream>().unwrap(),
        types[1].parse::<TokenStream>().unwrap(),
    )
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
    let (args, ret) = get_types();
    let (patterns, types) = args_divide(&args);

    let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
        env!("INPUTS_DIR"),
        "/default_public_input.toml"
    )))
    .unwrap();
    let public_types = args_divide_public(&args, &public_inputs.keys().collect())
        .0
         .1
        .iter()
        .map(|x| x.to_string() + ", ")
        .collect::<String>();
    let output_type = format!("pub type Output = ({} {});", public_types, ret).to_string();

    let (public_args, private_args) = args_split_public(&args, &public_inputs.keys().collect());
    let public_attrs = public_args
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let public_input_type =
        format!("{} pub struct PublicInput {{ {} }}", DERIVES, public_attrs).to_string();

    let private_attrs = private_args
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let private_input_type = format!(
        "{} pub struct PrivateInput {{ {} }}",
        DERIVES, private_attrs
    )
    .to_string();

    let all_args = args_split(&args);

    let mut struct_def = format!("{} pub struct Input {{", DERIVES);
    for arg in all_args {
        struct_def += &format!("pub {arg},");
    }

    let types = group_streams(&types);
    struct_def += &format!(
        "}}
        impl From<Input> for {types} {{
            fn from(input: Input) -> {types} {{
                (
    "
    );

    for field in patterns {
        struct_def += &format!("input.{field},");
    }
    struct_def += ") } }";

    (output_type + &public_input_type + &private_input_type + &struct_def)
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
    let (args, _) = get_types();
    let arg_patterns = args_divide(&args).0;

    foreach_field(item, arg_patterns)
}

/// Repeats the given code as many times as fields there are in the
/// PublicInput struct, while replacing all `.yield` occurences with the
/// concrete field name.
#[proc_macro]
pub fn foreach_public_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();

    let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
        env!("INPUTS_DIR"),
        "/default_public_input.toml"
    )))
    .unwrap();
    let public_patterns = args_divide_public(&args, &public_inputs.keys().collect())
        .0
         .0;

    foreach_field(item, public_patterns)
}

/// Repeats the given code as many times as fields there are in the
/// PrivateInput struct, while replacing all `.yield` occurences with the
/// concrete field name.
#[proc_macro]
pub fn foreach_private_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();

    let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
        env!("INPUTS_DIR"),
        "/default_public_input.toml"
    )))
    .unwrap();
    let private_patterns = args_divide_public(&args, &public_inputs.keys().collect())
        .1
         .0;

    foreach_field(item, private_patterns)
}

/// Assuming the `run_info` variable is present, it creates a block with all
/// needed code to properly benchmark the input code, according to all command
/// parameters.
#[proc_macro]
pub fn benchmarkable(item: TokenStream) -> TokenStream {
    format!(r#"
        {{
             use std::time::{{Duration, Instant}};
             use std::fs::OpenOptions;
             use std::io::Write;
             use std::path::Path;
             
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
                 let info_row = format!("name,guest,total duration,repeats,average\n");
                 let mut output = format!("{{}},{{}},", env!("ZKVM"), env!("GUEST"));

                 let duration = *ends.last().unwrap() - *starts.first().unwrap();
                 if run_info.millis {{ 
                    output += &format!("{{}},", duration.as_millis());
                 }} else {{ 
                    output += &format!("{{:.3}},", duration.as_secs_f32());
                 }}
                 
                 let durations = starts
                     .into_iter()
                     .zip(ends.into_iter())
                     .map(|(s,e)| e - s )
                     .collect::<Vec<Duration>>();
                 let average = durations.iter().sum::<Duration>() / durations.len() as u32;

                 if run_info.millis {{ 
                    output += &format!("{{}},{{}}\n", run_info.repeats, average.as_millis());
                 }} else {{ 
                    output += &format!("{{}},{{:.3}}\n", run_info.repeats, average.as_secs_f32());
                 }}

                 if let Some(file) = run_info.output_file {{

                    let file_exists = Path::new(&file).exists();

                    let mut outfile = match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .append(run_info.append)
                        .open(&file)
                    {{
                        Ok(file) => file,
                        Err(e) => {{
                            panic!("Failed to open file: {{}}", e);
                        }}
                    }};

                    if !file_exists {{
                        if let Err(e) = write!(outfile, "{{}}", info_row) {{
                            panic!("Failed to write info_row: {{}}", e);
                        }}
                    }}

                    if let Err(e) = write!(outfile, "{{}}", output) {{
                        panic!("Failed to write output: {{}}", e);
                    }}
                 }}
                 else {{
                     print!("{{}}", output);
                 }}
             }}
        }}
    "#).parse().unwrap()
}
