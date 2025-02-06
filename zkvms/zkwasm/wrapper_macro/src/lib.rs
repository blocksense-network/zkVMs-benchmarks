use proc_macro::{ TokenStream, TokenTree, Ident };

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_divide_grouped, args_divide_public, group_streams };
use toml::Table;

fn insert_reads(out: &mut TokenStream, patterns: &Vec<TokenStream>, types: &Vec<TokenStream>, readfn: &str) {
    for i in 0..patterns.len() {
        let type_note: String = format!("{}", types[i])
            .replace('<', "[")
            .replace('>', "]");
        out.extend(format!("let {} : {} = read!({} {});", patterns[i], types[i], readfn, type_note).parse::<TokenStream>());
    }
}

#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let (name, args, ret) = split_fn(&item);

    let public_inputs = toml::from_str::<Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"))
        )
        .unwrap();
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = args_divide_public(&args, &public_inputs.keys().collect());

    let mut out = TokenStream::new();

    insert_reads(&mut out, &pub_pat, &pub_typ, "read_public");
    insert_reads(&mut out, &prv_pat, &prv_typ, "read_private");

    let (ts_patterns, _) = args_divide_grouped(&args);

    out.extend(format!("
        let result = zkp::{}{};
        assert(result);
        write(result as u64);
    ", name, ts_patterns).parse::<TokenStream>());

    let mut block = TokenStream::new();
    block.extend(format!("{{ {} }}", out).parse::<TokenStream>());
    block
}

fn return_primitive(readfn: &TokenTree, typ: &Ident) -> TokenStream {
    format!("
        ({readfn}() as {typ})
    ").parse().unwrap()
}

fn return_bool(readfn: &TokenTree) -> TokenStream {
    format!("
        ({readfn}() != 0)
    ").parse().unwrap()
}

fn return_char(readfn: &TokenTree) -> TokenStream {
    format!("
        (({readfn}() as u8) as char)
    ").parse().unwrap()
}

fn return_string(readfn: &TokenTree) -> TokenStream {
    format!("
        {{
             let mut ret = Vec::new();
             let size = read!({readfn} usize);
             for _ in 0..size {{
                 ret.push(read!({readfn} char));
             }}
             ret.into_iter().collect()
        }}
    ").parse().unwrap()
}

fn return_array(readfn: &TokenTree, inner: &TokenStream) -> TokenStream {
    format!("
        {{
             let mut ret = Vec::new();
             let size = read!({readfn} usize);
             for _ in 0..size {{
                 ret.push(read!({readfn} {inner}));
             }}
             ret.try_into().unwrap()
        }}
    ").parse().unwrap()
}

fn return_vec(readfn: &TokenTree, inner: &TokenStream) -> TokenStream {
    format!("
        {{
             let mut ret = Vec::new();
             let size = read!({readfn} usize);
             for _ in 0..size {{
                 ret.push(read!({readfn} {inner}));
             }}
             ret
        }}
    ").parse().unwrap()
}

fn return_hashmap(readfn: &TokenTree, inner: &TokenStream) -> TokenStream {
    let mut inner = inner.clone().into_iter();
    let key_type = inner.next().unwrap();
    inner.next().unwrap();
    let value_type = inner.next().unwrap();
    format!(r#"
        {{
             let mut ret = HashMap::new();
             let size = read!({readfn} usize);
             for _ in 0..size {{
                 ret.insert(read!({readfn} {key_type}), read!({readfn} {value_type}));
             }}
             ret
        }}
    "#).parse().unwrap()
}

fn return_tuple(readfn: &TokenTree, inner: &TokenStream) -> TokenStream {
    let mut value = String::new();
    for subtype in inner.clone().into_iter() {
        value += &format!("read!({readfn} {subtype}), ");
    }
    format!("
        {{
             let _ = read!({readfn} usize);
             ( {value} )
        }}
    ").parse().unwrap()
}

#[proc_macro]
pub fn read(item: TokenStream) -> TokenStream {
    let mut parts = item.clone().into_iter();
    let readfn = parts.next().unwrap();
    match parts.next().unwrap() {
        // Primitive or STD Container
        TokenTree::Ident(ident) => {
            match ident.to_string().as_str() {
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" |
                "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
                "f32" | "f64" =>
                    return return_primitive(&readfn, &ident),
                "char" =>
                    return return_char(&readfn),
                "bool" =>
                    return return_bool(&readfn),
                "String" =>
                    return return_string(&readfn),
                _ => {},
            }

            let mut group = parts.next()
                .expect(format!("No group after \"{ident}\" while parsing \"{item}\"!").as_str());
            if let TokenTree::Group(inner_group) = group {
                let rest = inner_group.stream();

                match ident.to_string().as_str() {
                    "Vec" => return_vec(&readfn, &rest),
                    "HashMap" => return_hashmap(&readfn, &rest),
                    _ => todo!("Unsupported container {ident}"),
                }
            }
            else {
                unreachable!("{group} is not a TokenTree::Group!");
            }
        },
        // Array or tuple
        TokenTree::Group(group) => {
            let mut group = group.stream().into_iter();
            let mut inner = TokenStream::new();
            while let Some(current) = group.next() {
                match current {
                    TokenTree::Punct(punct) => match punct.as_char() {
                        // Array
                        ';' =>
                            return return_array(&readfn, &inner),
                        // Tuple
                        ',' => continue,
                        _ => unreachable!("Group contains unexpected \"{punct}\""),
                    },
                    TokenTree::Ident(_) | TokenTree::Group(_) =>
                        inner.extend([current].into_iter()),
                    _ => unreachable!(),
                }
            }
            return_tuple(&readfn, &inner)
        },
        _ => unreachable!(),
    }
}
