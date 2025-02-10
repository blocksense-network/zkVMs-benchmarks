//! A "library" with procedural macro functions for parsing and handling
//! function definitions.
//!
//! As of writing, exporting procedural macro functions is not supported.
//! Therefore, it should be used in ZKVM wrapper_macro crates, via a [mod path
//! attribute](https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute).

use proc_macro::{ TokenStream, TokenTree, Delimiter, Spacing, Group };

/// Split function definition into triplet of name, arguments and output types.
///
/// **Input:**  "fn name(...) -> ... { ..... }"
/// **Output:** "name", "(...)", "..."
pub fn split_fn(item: &TokenStream) -> (TokenStream, TokenStream, TokenStream) {
    let item = item.clone().into_iter();

    let mut name = TokenStream::new();
    let mut args = TokenStream::new();
    let mut ret  = TokenStream::new();
    let mut out: &mut TokenStream = &mut name;

    for tt in item {
        match tt {
            // The conditions will later be used to return
            // errors when incorrect function type is used
            TokenTree::Ident(ref ident) => {
                if ident.to_string() == "fn" || ident.to_string() == "pub" {
                    continue;
                }
            },
            TokenTree::Punct(ref punct) => {
                if punct.as_char() == '-' {
                    out = &mut ret;
                    continue;
                }
                if punct.as_char() == '>' && out.is_empty() {
                    continue;
                }
            },
            TokenTree::Group(ref group) => {
                if group.delimiter() == Delimiter::Brace {
                    break;
                }
                if ! out.is_empty() {
                    out = &mut args;
                }
            },
            TokenTree::Literal(_) => unreachable!("Cannot have literal inside def!"),
        }
        out.extend([tt].into_iter());
    }

    (name, args, ret)
}

/// Split arguments group into a vector of each argument with it's associated
/// type.
///
/// **Input:**  "(p1 : t1, p2 : t2, ...)"
/// **Output:** vec!["p1 : t1", "p2 : t2", ...]
pub fn args_split(item: &TokenStream) -> Vec<TokenStream> {
    let contents;
    if let TokenTree::Group(group) = item.clone().into_iter().next().unwrap() {
        contents = group.stream().into_iter();
    }
    else {
        unreachable!("Item passed to args_split is not a group: \"{item}\"");
    }

    let mut args = Vec::new();
    let mut ts = TokenStream::new();
    let mut angle_level = 0;

    for tt in contents {
        match tt {
            TokenTree::Punct(ref punct) => match punct.as_char() {
                    // < and > do **not** form TokenTree groups, however their
                    // usage is like that of a group. Hence, we need extra
                    // logic to skip them.
                    '<' => angle_level += 1,
                    '>' => angle_level -= 1,
                    ',' => if angle_level == 0 {
                        args.push(ts);
                        ts = TokenStream::new();
                        continue;
                    },
                    _ => {},
                },
            _ => {},
        }

        ts.extend([tt].into_iter());
    }

    if ! ts.is_empty() {
        args.push(ts);
    }
    args
}

/// Like `args_split`, however two vectors are returned: the first for public
/// arguments (and their types) and the second for private ones.
///
/// `public` is a vector of argument names.
///
/// **Input:**  "(p1 : t1, p2: t2, ...)", vec!["p3", "p4", ...]
/// **Output:** vec!["p1 : t1", "p2: t2", ...], vec!["p3 : t3", "p4: t4", ...]
pub fn args_split_public(item: &TokenStream, public: &Vec<&String>) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let all_args = args_split(item);
    let public_args: Vec<TokenStream> = all_args
        .clone()
        .into_iter()
        .filter(|a| public.iter().any(|x| a.to_string().starts_with(*x)))
        .collect();
    let private_args: Vec<TokenStream> = all_args
        .into_iter()
        .filter(|t| !public_args.iter().any(|pt| *t.to_string() == pt.to_string()))
        .collect();
    (public_args, private_args)
}

/// Split arguments group into two vectors: one for all argument names and one
/// for every argument type.
///
/// **Input:**  "(p1 : t1, p2: t2, ...)"
/// **Output:** vec!["p1", "p2", ...], vec!["t1", "t2", ...]
pub fn args_divide(item: &TokenStream) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let contents;
    if let TokenTree::Group(group) = item.clone().into_iter().next().unwrap() {
        contents = group.stream().into_iter();
    }
    else {
        unreachable!("Item passed to args_divide is not a group: \"{item}\"");
    }

    let mut patterns = Vec::new();
    let mut types = Vec::new();
    let mut ts = TokenStream::new();
    let mut ignore_next = false;
    let mut angle_level = 0;

    for tt in contents {
        match tt {
            TokenTree::Punct(ref punct) => {
                // Ignore "::"
                if punct.spacing() == Spacing::Joint && punct.as_char() == ':' {
                    ignore_next = true;
                }
                else if !ignore_next {
                    match punct.as_char() {
                        // < and > do **not** form TokenTree groups, however their
                        // usage is like that of a group. Hence, we need extra
                        // logic to skip them.
                        '<' => angle_level += 1,
                        '>' => angle_level -= 1,
                        ':' => {
                            patterns.push(ts);
                            ts = TokenStream::new();
                            continue;
                        },
                        ',' => if angle_level == 0 {
                            types.push(ts);
                            ts = TokenStream::new();
                            continue;
                        },
                        _ => {},
                    }
                }
                else {
                    ignore_next = false;
                }
            },
            _ => {},
        }

        ts.extend([tt].into_iter());
    }

    types.push(ts);
    (patterns, types)
}

/// Like `args_divide`, however two tuples of vectors are returned: the first
/// for public arguments and types, and the second for private ones.
///
/// `public` is a vector of argument names.
///
/// **Input:**  "(p1 : t1, p2: t2, ...)", vec!["p3", "p4", ...]
/// **Output:** (vec!["p1", "p2", ...], vec!["t1", "t2", ...]), (vec!["p3", "p4", ...], vec!["t3", "t4", ...])
pub fn args_divide_public(item: &TokenStream, public: &Vec<&String>) -> ((Vec<TokenStream>, Vec<TokenStream>), (Vec<TokenStream>, Vec<TokenStream>)) {
    let (patterns, types) = args_divide(item);

    let (public_patterns, public_types): (Vec<TokenStream>, Vec<TokenStream>) = patterns
        .clone()
        .into_iter()
        .zip(types.clone().into_iter())
        .filter(|(p, _)| public.iter().any(|x| p.to_string() == **x))
        .unzip();

    let (private_patterns, private_types): (Vec<TokenStream>, Vec<TokenStream>) = patterns
        .into_iter()
        .zip(types.into_iter())
        .filter(|(p, _)| !public_patterns.iter().any(|x| p.to_string() == x.to_string()))
        .unzip();
    ((public_patterns, public_types), (private_patterns, private_types))
}

/// Like `args_divide`, but group arguments and types (via `group_streams`).
///
/// **Input:**  "(p1 : t1, p2: t2, ...)"
/// **Output:** "(p1, p2, ...)", "(t1, t2, ...)"
pub fn args_divide_grouped(item: &TokenStream) -> (TokenStream, TokenStream) {
    let (patterns, types) = args_divide(&item);
    (group_streams(&patterns), group_streams(&types))
}

/// Transform a vector of elements into a (TokenTree) group of elements
///
/// **Input:**  vec!["p1", "p2", ...]
/// **Output:** "(p1, p2, ...)"
pub fn group_streams(patterns: &Vec<TokenStream>) -> TokenStream {
    let mut inner_ts = TokenStream::new();
    inner_ts.extend(patterns.clone().into_iter().flat_map(|i| [",".parse().unwrap(), i]).skip(1));

    let mut out = TokenStream::new();
    out.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, inner_ts))].into_iter());

    out
}
