use proc_macro::TokenStream;

#[path = "../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ args_split, args_divide, group_streams };


fn get_args() -> TokenStream {
    "(graph: Vec<Vec<bool>>, colors: u32, coloring: Vec<Vec<u32>>,)".parse::<TokenStream>().unwrap()
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
