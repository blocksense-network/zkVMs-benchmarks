use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ args_split, args_divide, group_streams };

fn get_args() -> TokenStream {
    include_str!("../../../guests/type.txt").parse::<TokenStream>().unwrap()
}

#[proc_macro]
pub fn generate_input_struct(_: TokenStream) -> TokenStream {
    let args = &get_args();
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

    struct_def.parse::<TokenStream>().unwrap()
}

#[proc_macro]
pub fn foreach_input_field(item: TokenStream) -> TokenStream {
    let arg_patterns = args_divide(&get_args()).0;

    let expr = format!("{}", item);
    let mut out = String::new();
    for field in arg_patterns {
        // Unquoted yield is a keyword, so it is not allowed as field name
        out += &expr.replace(".yield", &format!(".{field}"));
    }
    out.parse::<TokenStream>().unwrap()
}
