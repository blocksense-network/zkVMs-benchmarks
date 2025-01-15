use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_split, args_divide, group_streams };

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);
    let (patterns, types) = args_divide(&args);

    let mut out = TokenStream::new();

    for i in 0..patterns.len() {
        let type_note: String = format!("{}", types[i])
            .chars()
            .map(|c| match c {
                '<' => ',',
                '>' => ' ',
                 _  => c,
            })
            .collect();
        out.extend(format!("let {} : {} = read!({});", patterns[i], types[i], type_note).parse::<TokenStream>());
    }

    let ts_patterns = group_streams(&patterns);

    out.extend(format!("let result = zkp::{}{}; assert(result); write(result as u64);", name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
