use proc_macro::TokenStream;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_divide_grouped, args_divide_public, group_streams };

fn insert_reads(out: &mut TokenStream, patterns: &Vec<TokenStream>, types: &Vec<TokenStream>, readfn: &str) {
    for i in 0..patterns.len() {
        let type_note: String = format!("{}", types[i])
            .chars()
            .map(|c| match c {
                '<' => ',',
                '>' => ' ',
                 _  => c,
            })
            .collect();
        out.extend(format!("let {} : {} = read!({} {});", patterns[i], types[i], type_note, readfn).parse::<TokenStream>());
    }
}

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let public_inputs = toml::from_str::<toml::Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = args_divide_public(&args, &public_inputs.keys().collect());

    let mut out = TokenStream::new();

    insert_reads(&mut out, &pub_pat, &pub_typ, "read_public");
    insert_reads(&mut out, &prv_pat, &prv_typ, "read_private");

    let (ts_patterns, _) = args_divide_grouped(&args);

    out.extend(format!("let result = zkp::{}{}; assert(result); write(result as u64);", name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}
