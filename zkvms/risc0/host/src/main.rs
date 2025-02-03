use zkvms_host_io::{Input, Output, foreach_input_field, read_args, RunType::{ Execute, Prove, Verify }};
use risc0_zkvm::{default_prover, default_executor, ExecutorEnv, Receipt};
use risc0_zkp::core::digest::Digest;
use hex::FromHex;

// https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L280-L284
static HELLO_GUEST_ELF: &[u8] = include_bytes!("./guest");
// https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L255
static HELLO_GUEST_ID: &str = env!("GUEST_ID");

fn build_env(input: &Input) -> ExecutorEnv {
    let mut builder = ExecutorEnv::builder();
    foreach_input_field!{
        builder.write(&input.yield).unwrap();
    }
    builder.build().unwrap()
}

fn prove(env: ExecutorEnv) -> Receipt {
    default_prover()
        .prove(env, HELLO_GUEST_ELF)
        .expect("Error occured")
        .receipt
}

fn journal(receipt: Receipt) -> Output {
    receipt
        .journal
        .decode()
        .unwrap()
}

fn main() {
    let run_info = read_args();
    let env = build_env(&run_info.input);

    match run_info.run_type {
        Execute => {
            let exec = default_executor();
            let output = default_executor()
                .execute(env, HELLO_GUEST_ELF)
                .unwrap()
                .receipt_claim
                .unwrap()
                .output
                .value()
                .unwrap();

            println!("{:#?}", output);
        },
        Prove => {
            let receipt = prove(env);
            println!("Output from journal: {:?}", journal(receipt));
        },
        Verify => {
            // https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L197-L199
            let guest_id: Digest = Digest::from_hex(HELLO_GUEST_ID).unwrap();

            let receipt = prove(env);
            receipt.verify(guest_id).unwrap();

            println!("Output from verify: {:?}", journal(receipt));
        },
    }
}

