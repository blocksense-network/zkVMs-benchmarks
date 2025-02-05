use std::{ fs::File, io::Write };
use proc_macro::TokenStream;
mod parse_fn;

#[proc_macro_attribute]
pub fn proving_entrypoint(_: TokenStream, mut item: TokenStream) -> TokenStream {
    let (name, args, ret) = parse_fn::split_fn(&item);

    // Put the file in zkVMs-benchmarks/guests/
    let mut output = File::create("../type.txt").unwrap();
    writeln!(output, "{}", &format!("{args}").replace('\n', " "));
    write!(output, "{}", &format!("{ret}").replace('\n', " "));

    item.extend(format!("#[macro_export]
        macro_rules! entrypoint_expr {{
            () => {{
                make_wrapper!{{{}{} -> {}}}
            }};
        }}", name, args, ret).parse::<TokenStream>());
    item
}
