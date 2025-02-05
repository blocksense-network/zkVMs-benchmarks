use proc_macro::{ TokenStream, TokenTree, Ident };

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::{ split_fn, args_divide_grouped, args_divide_public, group_streams };
use toml::Table;

fn insert_reads(out: &mut TokenStream, patterns: &Vec<TokenStream>, types: &Vec<TokenStream>, inputs: &Table, readfn: &str) {
    for i in 0..patterns.len() {
        let mut value = &inputs[&patterns[i].to_string()];
        let type_note: String = format!("{}", types[i])
            .replace('<', "[")
            .replace('>', "]")
            .split("[")
            .map(|x| x.trim())
            .map(|typ|
                // Array
                if typ.is_empty() {
                    let array = value.as_array()
                        .expect("value is of type Array but isn't an array");
                    value = &array[0];
                    "[".to_string()
                }
                // STD Vec
                else if typ.ends_with("Vec") {
                    let array = value.as_array()
                        .expect("value is of type Vec but isn't an array");
                    value = &array[0];
                    format!("{} [ {} ", typ, array.len())
                }
                else {
                    typ.to_string()
                })
            .collect();
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
    let private_inputs = toml::from_str::<Table>(
            include_str!(concat!(env!("INPUTS_DIR"), "/default_private_input.toml"))
        )
        .unwrap();
    let ((pub_pat, pub_typ), (prv_pat, prv_typ)) = args_divide_public(&args, &public_inputs.keys().collect());

    let mut out = TokenStream::new();

    insert_reads(&mut out, &pub_pat, &pub_typ, &public_inputs, "read_public");
    insert_reads(&mut out, &prv_pat, &prv_typ, &private_inputs, "read_private");

    let (ts_patterns, _) = args_divide_grouped(&args);

    out.extend(format!("let result = zkp::{}{}; assert(result); write(result as u64);", name, ts_patterns).parse::<TokenStream>());

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
             let mut current_char = read!({readfn} char);
             while current_char != '\\0' {{
                 ret.push(current_char);
                 current_char = read!({readfn} char);
             }}
             ret.into_iter().collect()
        }}
    ").parse().unwrap()
}

fn return_array(readfn: &TokenTree, size: &TokenTree, inner: &TokenStream) -> TokenStream {
    format!("
        {{
             let mut ret = Vec::new();
             for _ in 0..{size} {{
                 ret.push(read!({readfn} {inner}));
             }}
             ret.try_into().unwrap()
        }}
    ").parse().unwrap()
}

fn return_vec(readfn: &TokenTree, size: &TokenTree, inner: &TokenStream) -> TokenStream {
    format!("
        {{
             let mut ret = Vec::new();
             for _ in 0..{size} {{
                 ret.push(read!({readfn} {inner}));
             }}
             ret
        }}
    ").parse().unwrap()
}

fn return_tuple(readfn: &TokenTree, inner: &TokenStream) -> TokenStream {
    let mut value = String::new();
    for subtype in inner.clone().into_iter() {
        value += &format!("read!({readfn} {subtype}), ");
    }
    format!("( {value} )").parse().unwrap()
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
                let mut group = inner_group.stream().into_iter();
                let size = group.next().unwrap();

                let mut rest = TokenStream::new();
                rest.extend(group);

                match ident.to_string().as_str() {
                    "Vec" => return_vec(&readfn, &size, &rest),
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
                        ';' => {
                            let size = group.next().unwrap();
                            return return_array(&readfn, &size, &inner);
                        },
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
