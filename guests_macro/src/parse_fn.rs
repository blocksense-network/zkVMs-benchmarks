//! A "library" with procedural macro functions for parsing and handling
//! function definitions.
//!
//! As of writing, exporting procedural macro functions is not supported.
//! Therefore, it should be used in ZKVM wrapper_macro crates, via a [mod path
//! attribute](https://doc.rust-lang.org/reference/items/modules.html#the-path-attribute).

use proc_macro::{Delimiter, Group, Spacing, TokenStream, TokenTree};

pub struct FunctionDefinition {
    pub name: TokenStream,
    pub args: TokenStream,
    pub return_type: TokenStream,

    patterns: Vec<TokenStream>,
    types: Vec<TokenStream>,

    public_patterns: Vec<TokenStream>,
    public_types: Vec<TokenStream>,

    private_patterns: Vec<TokenStream>,
    private_types: Vec<TokenStream>,
}

impl FunctionDefinition {
    pub fn new(item: &TokenStream) -> FunctionDefinition {
        let (name, args, return_type) = Self::split_fn(item);
        let (patterns, types) = Self::args_divide(&args);

        let public_inputs = toml::from_str::<toml::Table>(include_str!(concat!(
            env!("INPUTS_DIR"),
            "/default_public_input.toml"
        )))
        .unwrap();
        let ((public_patterns, public_types), (private_patterns, private_types)) =
            Self::args_divide_public(&patterns, &types, &public_inputs.keys().collect());

        FunctionDefinition {
            name,
            args,
            return_type,
            patterns,
            types,
            public_patterns,
            public_types,
            private_patterns,
            private_types,
        }
    }

    pub fn patterns(&self) -> &Vec<TokenStream> {
        &self.patterns
    }
    pub fn public_patterns(&self) -> &Vec<TokenStream> {
        &self.public_patterns
    }
    pub fn private_patterns(&self) -> &Vec<TokenStream> {
        &self.private_patterns
    }

    pub fn types(&self) -> &Vec<TokenStream> {
        &self.types
    }
    pub fn public_types(&self) -> &Vec<TokenStream> {
        &self.public_types
    }
    pub fn private_types(&self) -> &Vec<TokenStream> {
        &self.private_types
    }

    pub fn grouped_patterns(&self) -> TokenStream {
        Self::group_stream(&self.patterns)
    }
    pub fn grouped_public_patterns(&self) -> TokenStream {
        Self::group_stream(&self.public_patterns)
    }
    pub fn grouped_private_patterns(&self) -> TokenStream {
        Self::group_stream(&self.private_patterns)
    }

    pub fn grouped_types(&self) -> TokenStream {
        Self::group_stream(&self.types)
    }
    pub fn grouped_public_types(&self) -> TokenStream {
        Self::group_stream(&self.public_types)
    }
    pub fn grouped_private_types(&self) -> TokenStream {
        Self::group_stream(&self.private_types)
    }

    pub fn arguments(&self) -> Vec<TokenStream> {
        Self::combine(self.patterns.clone(), self.types.clone())
    }
    pub fn public_arguments(&self) -> Vec<TokenStream> {
        Self::combine(self.public_patterns.clone(), self.public_types.clone())
    }
    pub fn private_arguments(&self) -> Vec<TokenStream> {
        Self::combine(self.private_patterns.clone(), self.private_types.clone())
    }

    /// Split function definition into triplet of name, arguments and output types.
    ///
    /// **Input:**  "fn name(...) -> ... { ..... }"
    /// **Output:** "name", "(...)", "..."
    fn split_fn(item: &TokenStream) -> (TokenStream, TokenStream, TokenStream) {
        let item = item.clone().into_iter();

        let mut name = TokenStream::new();
        let mut args = TokenStream::new();
        let mut ret = TokenStream::new();
        let mut out: &mut TokenStream = &mut name;

        for tt in item {
            match tt {
                // The conditions will later be used to return
                // errors when incorrect function type is used
                TokenTree::Ident(ref ident) => {
                    if ident.to_string() == "fn" || ident.to_string() == "pub" {
                        continue;
                    }
                }
                TokenTree::Punct(ref punct) => {
                    if punct.as_char() == '-' {
                        out = &mut ret;
                        continue;
                    }
                    if punct.as_char() == '>' && out.is_empty() {
                        continue;
                    }
                }
                TokenTree::Group(ref group) => {
                    if group.delimiter() == Delimiter::Brace {
                        break;
                    }
                    if !out.is_empty() {
                        out = &mut args;
                    }
                }
                TokenTree::Literal(_) => unreachable!("Cannot have literal inside def!"),
            }
            out.extend([tt].into_iter());
        }

        if ret.is_empty() {
            ret.extend(
                [TokenTree::Group(Group::new(
                    Delimiter::Parenthesis,
                    TokenStream::new(),
                ))]
                .into_iter(),
            );
        }

        (name, args, ret)
    }

    /// Split arguments group into two vectors: one for all argument names and one
    /// for every argument type.
    ///
    /// **Input:**  "(p1 : t1, p2: t2, ...)"
    /// **Output:** vec!["p1", "p2", ...], vec!["t1", "t2", ...]
    fn args_divide(item: &TokenStream) -> (Vec<TokenStream>, Vec<TokenStream>) {
        let contents;
        if let TokenTree::Group(group) = item.clone().into_iter().next().unwrap() {
            contents = group.stream().into_iter();
        } else {
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
                    } else if !ignore_next {
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
                            }
                            ',' => {
                                if angle_level == 0 {
                                    types.push(ts);
                                    ts = TokenStream::new();
                                    continue;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        ignore_next = false;
                    }
                }
                _ => {}
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
    fn args_divide_public(
        patterns: &Vec<TokenStream>,
        types: &Vec<TokenStream>,
        public: &Vec<&String>,
    ) -> (
        (Vec<TokenStream>, Vec<TokenStream>),
        (Vec<TokenStream>, Vec<TokenStream>),
    ) {
        let (public_patterns, public_types): (Vec<TokenStream>, Vec<TokenStream>) = patterns
            .clone()
            .into_iter()
            .zip(types.clone().into_iter())
            .filter(|(p, _)| public.iter().any(|x| p.to_string() == **x))
            .unzip();

        let (private_patterns, private_types): (Vec<TokenStream>, Vec<TokenStream>) = patterns
            .clone()
            .into_iter()
            .zip(types.clone().into_iter())
            .filter(|(p, _)| {
                !public_patterns
                    .iter()
                    .any(|x| p.to_string() == x.to_string())
            })
            .unzip();
        (
            (public_patterns, public_types),
            (private_patterns, private_types),
        )
    }

    /// Transform a vector of elements into a (TokenTree) group of elements
    ///
    /// **Input:**  vec!["p1", "p2", ...]
    /// **Output:** "p1, p2, ..."
    fn group_stream(patterns: &Vec<TokenStream>) -> TokenStream {
        let mut elems = TokenStream::new();
        elems.extend(
            patterns
                .clone()
                .into_iter()
                .flat_map(|i| [",".parse().unwrap(), i])
                .skip(1),
        );
        elems
    }

    fn combine(patterns: Vec<TokenStream>, types: Vec<TokenStream>) -> Vec<TokenStream> {
        patterns
            .into_iter()
            .zip(types.into_iter())
            .map(|(p, t)| format!("{p} : {t}").parse().unwrap())
            .collect()
    }
}
