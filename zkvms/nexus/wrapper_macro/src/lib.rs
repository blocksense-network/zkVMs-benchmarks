use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide_public, args_divide_grouped };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let (ts_patterns, ts_types) = args_divide_grouped(&args);

    let mut out = TokenStream::new();
    out.extend(format!("let {} = read_private_input::<{}>().unwrap();", ts_patterns, ts_types).parse::<TokenStream>());

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let (public_patterns, public_types) = args_divide_public(&args, &public_inputs.keys().collect()).0;
    let public_patterns: Vec<(TokenStream, TokenStream)> = public_patterns
        .into_iter()
        .zip(public_types.into_iter())
        .collect();
    for (pattern, ptype) in public_patterns {
        out.extend(format!("write_output::<{}>(&{});", ptype, pattern).parse::<TokenStream>());
    }

    out.extend(format!("write_output::<{}>(&zkp::{}{});", ret, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
