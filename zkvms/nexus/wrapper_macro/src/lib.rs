use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide_grouped };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let (ts_patterns, ts_types) = args_divide_grouped(&args);

    let mut out = TokenStream::new();
    out.extend(format!("let {} = read_private_input::<{}>().unwrap();", ts_patterns, ts_types).parse::<TokenStream>());
    out.extend(format!("write_output::<{}>(&zkp::{}{});", ret, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
