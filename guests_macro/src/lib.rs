use proc_macro::TokenStream;
mod parse_fn;

#[proc_macro_attribute]
pub fn proving_entrypoint(_: TokenStream, mut item: TokenStream) -> TokenStream {
    let (name, args, ret) = parse_fn::split_fn(&item);
    item.extend(format!("#[macro_export] macro_rules! entrypoint_expr {{ () => {{ make_wrapper!{{{}{} -> {}}} }}; }}", name, args, ret).parse::<TokenStream>());
    item
}
