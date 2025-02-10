use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ args_split, args_split_public, args_divide, args_divide_public, group_streams };

fn get_types() -> (TokenStream, TokenStream) {
    let types: Vec<&str> = include_str!("../../../guests/type.txt")
        .split('\n')
        .collect();
    (types[0].parse::<TokenStream>().unwrap(), types[1].parse::<TokenStream>().unwrap())
}

static DERIVES: &str = "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]";

#[proc_macro]
pub fn generate_output_type_input_struct(_: TokenStream) -> TokenStream {
    let (args, ret) = get_types();
    let (patterns, types) = args_divide(&args);

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
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
    let public_input_type = format!("{} pub struct PublicInput {{ {} }}", DERIVES, public_attrs).to_string();

    let private_attrs = private_args
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let private_input_type = format!("{} pub struct PrivateInput {{ {} }}", DERIVES, private_attrs).to_string();

    let all_args = args_split(&args);

    let mut struct_def = format!("{} pub struct Input {{", DERIVES);
    for arg in all_args {
        struct_def += &format!("pub {arg},");
    }

    let types = group_streams(&types);
    struct_def += &format!("}}
        impl From<Input> for {types} {{
            fn from(input: Input) -> {types} {{
                (
    ");

    for field in patterns {
        struct_def += &format!("input.{field},");
    }
    struct_def += ") } }";

    (output_type + &public_input_type + &private_input_type + &struct_def).parse::<TokenStream>().unwrap()
}

fn foreach_field(item: TokenStream, fields: Vec<TokenStream>) -> TokenStream {
    let expr = format!("{}", item);
    let mut out = String::new();
    for field in fields {
        // Unquoted yield is a keyword, so it is not allowed as field name
        out += &expr.replace(".yield", &format!(".{field}"));
    }
    out.parse::<TokenStream>().unwrap()
}

#[proc_macro]
pub fn foreach_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();
    let arg_patterns = args_divide(&args).0;

    foreach_field(item, arg_patterns)
}

#[proc_macro]
pub fn foreach_public_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let public_patterns = args_divide_public(&args, &public_inputs.keys().collect()).0.0;

    foreach_field(item, public_patterns)
}

#[proc_macro]
pub fn foreach_private_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let private_patterns = args_divide_public(&args, &public_inputs.keys().collect()).1.0;

    foreach_field(item, private_patterns)
}

#[proc_macro]
pub fn benchmarkable(item: TokenStream) -> TokenStream {
    format!(r#"
        {{
             use std::time::Instant;
             use std::fs::OpenOptions;
             use std::io::Write;

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
                 let mut output = format!("zkvm,{{}}\n", env!("ZKVM"));

                 let duration = *ends.last().unwrap() - *starts.first().unwrap();
                 let duration = if run_info.millis {{ duration.as_millis() }} else {{ duration.as_secs().into() }};
                 output += &format!("duration,{{duration}}\n");

                 let durations = starts
                     .into_iter()
                     .zip(ends.into_iter())
                     .map(|(s,e)| if run_info.millis {{ (e - s).as_millis() }} else {{ (e - s).as_secs().into() }})
                     .collect::<Vec<u128>>();
                 let average = durations.iter().sum::<u128>() / durations.len() as u128;
                 output += &format!("repeats,{{}}\naverage,{{average}}\n", run_info.repeats);

                 if let Some(file) = run_info.output_file {{
                     let mut outfile = OpenOptions::new()
                         .write(true)
                         .create(true)
                         .append(run_info.append)
                         .open(file)
                         .unwrap();
                     write!(outfile, "{{}}", output);
                 }}
                 else {{
                     print!("{{}}", output);
                 }}
             }}
        }}
    "#).parse().unwrap()
}
