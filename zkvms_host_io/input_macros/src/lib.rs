use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ args_split, args_divide, group_streams };

fn get_types() -> (TokenStream, TokenStream) {
    let types: Vec<&str> = include_str!("../../../guests/type.txt")
        .split('\n')
        .collect();
    (types[0].parse::<TokenStream>().unwrap(), types[1].parse::<TokenStream>().unwrap())
}

#[proc_macro]
pub fn generate_output_type_input_struct(_: TokenStream) -> TokenStream {
    let (args, ret) = get_types();
    let output_type = format!("pub type Output = {};", ret).to_string();

    let all_args = args_split(&args);

    let mut struct_def = "#[derive(Debug, Serialize, Deserialize)] pub struct Input {".to_string();
    for arg in all_args {
        struct_def += &format!("pub {arg},");
    }

    let (patterns, types) = args_divide(&args);
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

    (output_type + &struct_def).parse::<TokenStream>().unwrap()
}

#[proc_macro]
pub fn foreach_input_field(item: TokenStream) -> TokenStream {
    let (args, _) = get_types();
    let arg_patterns = args_divide(&args).0;

    let expr = format!("{}", item);
    let mut out = String::new();
    for field in arg_patterns {
        // Unquoted yield is a keyword, so it is not allowed as field name
        out += &expr.replace(".yield", &format!(".{field}"));
    }
    out.parse::<TokenStream>().unwrap()
}
