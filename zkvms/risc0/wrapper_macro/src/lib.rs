use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide_public, args_divide_grouped };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);
    let args_split = args_split(&args);

    let mut out = TokenStream::new();
    for arg in args_split {
        out.extend(format!("let {} = read();", arg).parse::<TokenStream>());
    }

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let public_patterns = args_divide_public(&args, &public_inputs.keys().collect())
        .0
        .iter()
        .map(|x| x.to_string() + ".clone(), ")
        .collect::<String>();

    let (ts_patterns, _) = args_divide_grouped(&args);

    out.extend(format!("commit(&({} zkp::{}{}));", public_patterns, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
