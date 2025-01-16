use proc_macro::{ TokenStream, TokenTree, Delimiter, Spacing, Group };

/// Input:  "fn name(...) -> ... { ... }"
/// Output: "name", "(...)", "..."
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

/// Input:  "(p1 : t1, p2: t2, ...)"
/// Output: "p1 : t1", "p2: t2", ...
pub fn args_split(item: &TokenStream) -> Vec<TokenStream> {
    let contents;
    if let TokenTree::Group(group) = item.clone().into_iter().next().unwrap() {
        contents = group.stream().into_iter();
    }
    else {
        unreachable!();
    }

    let mut args = Vec::new();
    let mut ts = TokenStream::new();

    for tt in contents {
        match tt {
            TokenTree::Punct(ref punct) =>
                if punct.as_char() == ',' {
                    args.push(ts);
                    ts = TokenStream::new();
                    continue;
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

/// Input:  (p1 : t1, p2: t2, ...)
/// Output: (p1, p2, ...), (t1, t2, ...)
pub fn args_divide(item: &TokenStream) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let contents;
    if let TokenTree::Group(group) = item.clone().into_iter().next().unwrap() {
        contents = group.stream().into_iter();
    }
    else {
        unreachable!();
    }

    let mut patterns = Vec::new();
    let mut types = Vec::new();
    let mut ts = TokenStream::new();
    let mut ignore_next = false;

    for tt in contents {
        match tt {
            TokenTree::Punct(ref punct) => {
                if punct.spacing() == Spacing::Joint {
                    ignore_next = true;
                }
                else if !ignore_next {
                    match punct.as_char() {
                        ':' => {
                            patterns.push(ts);
                            ts = TokenStream::new();
                            continue;
                        },
                        ',' => {
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

/// Input:  "p1 p2 ..."
/// Output: "(p1, p2, ...)"
pub fn group_streams(patterns: &Vec<TokenStream>) -> TokenStream {
    let mut inner_ts = TokenStream::new();
    inner_ts.extend(patterns.clone().into_iter().flat_map(|i| [",".parse().unwrap(), i]).skip(1));

    let mut out = TokenStream::new();
    out.extend([TokenTree::Group(Group::new(Delimiter::Parenthesis, inner_ts))].into_iter());

    out
}
