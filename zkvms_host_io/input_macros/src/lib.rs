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
        .1
        .iter()
        .map(|x| x.to_string() + ", ")
        .collect::<String>();
    let output_type = format!("pub type Output = ({} {});", public_types, ret).to_string();

    let all_args = args_split(&args);

    let public_args = args_split_public(&args, &public_inputs.keys().collect());
    let public_attrs = public_args
        .iter()
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let public_input_type = format!("{} pub struct PublicInput {{ {} }}", DERIVES, public_attrs).to_string();

    let private_attrs = all_args
        .iter()
        .filter(|t| !public_args.iter().any(|pt| *t.to_string() == pt.to_string()))
        .map(|x| format!("pub {x},"))
        .collect::<String>();
    let private_input_type = format!("{} pub struct PrivateInput {{ {} }}", DERIVES, private_attrs).to_string();

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
