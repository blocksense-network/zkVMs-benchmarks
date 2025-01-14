use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide, group_streams };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);
    let args_split = args_split(&args);

    let mut out = TokenStream::new();
    for arg in args_split {
        out.extend(format!("let {} = read();", arg).parse::<TokenStream>());
    }

    let (patterns, _) = args_divide(&args);
    let ts_patterns = group_streams(&patterns);

    out.extend(format!("commit(&zkp::{}{});", name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
