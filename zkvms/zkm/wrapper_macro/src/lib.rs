use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide_public, args_divide_grouped, group_streams };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let (ts_patterns, _) = args_divide_grouped(&args);

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = args_divide_public(&args, &public_inputs.keys().collect());
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = (
        (group_streams(&pub_pat), group_streams(&pub_typ)),
        (group_streams(&prv_pat), group_streams(&prv_typ)));

    let mut out = TokenStream::new();
    // NOTE: The first read returns public data, the second returns private
    out.extend(format!("let {} : {} = read();", pub_pat, pub_typ).parse::<TokenStream>());
    out.extend(format!("let {} : {} = read();", prv_pat, prv_typ).parse::<TokenStream>());

    out.extend(format!("commit::<{}>(&zkp::{}{});", ret, name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
