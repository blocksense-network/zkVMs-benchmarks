use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[path = "../../../../guests_macro/src/parse_fn.rs"]
mod parse_fn;
use crate::parse_fn::FunctionDefinition;

/// Create a set of three helper functions.
///
/// First a function called "guest" with the same arguments and return types
/// as our specific guest's entrypoint function, which calls the aforementioned
/// entrypoint function.
///
/// In essence, we create a wrapper function around the entrypoint, which also
/// has the required `jolt::provable` attribute.
///
/// Second is a `guests_closure` function, almost identical to Jolt's
/// `build_guest` function, which takes an already built guest ELF.
/// Since we're following Jolt's design, it uses a third generated function,
/// called `preprocess_guest_elf`.
///
/// # Usage
///
/// Inside Jolt's guest (excluding the `entrypoint_expr` call):
///
/// ```rust
/// make_wrapper!{fn main(...) -> ...}
/// ```
///
/// # Example output
///
/// ```rust
/// #[jolt::provable(max_input_size = 100000)]
/// fn guest(...) -> ... {
///     zkp::main(...)
/// }
///
/// #[cfg(all(not(target_arch = "wasm32"), not(feature = "guest")))]
/// pub fn guest_closures(elf_path: String) -> (...) { ..... }
///
/// #[cfg(all(not(target_arch = "wasm32"), not(feature = "guest")))]
/// pub fn preprocess_guest_elf(elf_path: String) -> (...) { ..... }
/// ```
#[proc_macro]
pub fn make_wrapper(item: TokenStream) -> TokenStream {
    let fd = FunctionDefinition::new(&item);

    let mut out = TokenStream::new();
    out.extend(format!("zkp::{}({})", fd.name, fd.grouped_patterns()).parse::<TokenStream>());

    let mut func = TokenStream::new();
    func.extend(
        format!(
            "#[jolt::provable(max_input_size = 100000)] fn guest{} -> {} {{ {} }}",
            fd.args, fd.return_type, out
        )
        .parse::<TokenStream>(),
    );

    func.extend(make_build_fn(
        fd.patterns().clone(),
        fd.types().clone(),
        fd.grouped_patterns().clone(),
        fd.grouped_types().clone(),
        fd.return_type.clone(),
    ));
    func.extend(make_preprocess_fn(
        fd.patterns().clone(),
        fd.types().clone(),
        fd.grouped_patterns().clone(),
        fd.grouped_types().clone(),
        fd.return_type.clone(),
    ));
    func
}

// Modified copies of
// https://github.com/a16z/jolt/blob/fa45507aaddb1815bafd54332e4b14173a7f8699/jolt-sdk/macros/src/lib.rs

fn make_build_fn(
    patterns: Vec<TokenStream>,
    types: Vec<TokenStream>,
    ts_patterns: TokenStream,
    ts_types: TokenStream,
    ret: TokenStream,
) -> TokenStream {
    let patterns = patterns.iter().map(|p| TokenStream2::from(p.clone()));
    let types = types.iter().map(|t| TokenStream2::from(t.clone()));
    let ts_patterns = TokenStream2::from(ts_patterns);
    let ts_types = TokenStream2::from(ts_types);
    let ret = if ret.is_empty() {
        quote! { () }
    } else {
        TokenStream2::from(ret)
    };

    let imports = make_imports();

    quote! {
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "guest")))]
        pub fn guest_closures(elf_path: String) -> (
            impl Fn((#ts_types)) -> (#ret, jolt::JoltHyperKZGProof) + Sync + Send,
            impl Fn(jolt::JoltHyperKZGProof) -> bool + Sync + Send
        ) {
            #imports
            let (program, preprocessing) = preprocess_guest_elf(elf_path);
            let program = std::sync::Arc::new(program);
            let preprocessing = std::sync::Arc::new(preprocessing);

            let program_cp = program.clone();
            let preprocessing_cp = preprocessing.clone();

            let prove_closure = move |args: (#ts_types)| {
                let program = (*program).clone();
                let preprocessing = (*preprocessing).clone();
                let (#ts_patterns) = args;
                prove_guest(program, preprocessing, #(#patterns),*)
            };


            let verify_closure = move |proof: jolt::JoltHyperKZGProof| {
                let program = (*program_cp).clone();
                let preprocessing = (*preprocessing_cp).clone();
                RV32IJoltVM::verify(preprocessing, proof.proof, proof.commitments, None).is_ok()
            };

            (prove_closure, verify_closure)
        }
    }
    .into()
}

fn make_preprocess_fn(
    patterns: Vec<TokenStream>,
    types: Vec<TokenStream>,
    ts_patterns: TokenStream,
    ts_types: TokenStream,
    ret: TokenStream,
) -> TokenStream {
    let patterns = patterns.iter().map(|p| TokenStream2::from(p.clone()));
    let types = types.iter().map(|t| TokenStream2::from(t.clone()));
    let ts_patterns = TokenStream2::from(ts_patterns);
    let ts_types = TokenStream2::from(ts_types);
    let ret = if ret.is_empty() {
        quote! { () }
    } else {
        TokenStream2::from(ret)
    };

    let imports = make_imports();

    quote! {
        #[cfg(all(not(target_arch = "wasm32"), not(feature = "guest")))]
        pub fn preprocess_guest_elf(elf_path: String) -> (
            jolt::host::Program,
            jolt::JoltPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript>
        ) {
            #imports
            use std::{ path::PathBuf, str::FromStr };

            let mut program = Program::new("guest");
            program.set_func("guest");
            program.elf = Some(PathBuf::from_str(&elf_path).unwrap());
            program.set_std(true);
            program.set_memory_size(10485760);
            program.set_stack_size(4096);
            program.set_max_input_size(100000u64);
            program.set_max_output_size(4096u64);
            let (bytecode, memory_init) = program.decode();
            let memory_layout = MemoryLayout::new(100000u64, 4096u64);

            let preprocessing: JoltPreprocessing<4, jolt::F, jolt::PCS, jolt::ProofTranscript> =
                RV32IJoltVM::preprocess(
                    bytecode,
                    memory_layout,
                    memory_init,
                    1 << 20,
                    1 << 20,
                    1 << 24
                );

            (program, preprocessing)
        }
    }
    .into()
}

fn make_imports() -> TokenStream2 {
    quote! {
        #[cfg(not(feature = "guest"))]
        use jolt::{
            JoltField,
            host::Program,
            JoltPreprocessing,
            Jolt,
            JoltCommitments,
            ProofTranscript,
            RV32IJoltVM,
            RV32I,
            RV32IJoltProof,
            BytecodeRow,
            MemoryOp,
            MemoryLayout,
            MEMORY_OPS_PER_INSTRUCTION,
            instruction::add::ADDInstruction,
            tracer,
        };
    }
}
