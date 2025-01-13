use risc0_zkvm::{default_prover, default_executor, ExecutorEnv};
use risc0_zkp::core::digest::Digest;
use hex::FromHex;

// https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L280-L284
static HELLO_GUEST_ELF: &[u8] = include_bytes!("./guest");
// https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L255
static HELLO_GUEST_ID: &str = env!("GUEST_ID");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let (graph, colors, coloring): (Vec<Vec<bool>>, u32, Vec<Vec<u32>>) = include!(env!("INPUTS"));

    let env = ExecutorEnv::builder()
        .write(&graph)
        .unwrap()
        .write(&colors)
        .unwrap()
        .write(&coloring)
        .unwrap()
        .build()
        .unwrap();

    match args[1].as_str() {
        "execute" => {
            let exec = default_executor();
            let output = exec.execute(env, HELLO_GUEST_ELF).unwrap().receipt_claim.unwrap().output.value().unwrap();

            println!("{:#?}", output);
        },
        "prove" => {
            // Obtain the default prover.
            let prover = default_prover();

            // Produce a receipt by proving the specified ELF binary.
            let receipt = prover.prove(env, HELLO_GUEST_ELF).expect("Error occured").receipt;

            // Extract journal of receipt
            let journal: bool = receipt.journal.decode().unwrap();
            // Print, notice, after committing to a journal, the private input became public
            println!("Output from journal: {:?}", journal);
        },
        "verify" => {
            // https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L197-L199
            let guest_id: Digest = Digest::from_hex(HELLO_GUEST_ID).expect("");
            // https://github.com/risc0/risc0/blob/881e512732eca72849b2d0e263a1242aba3158af/risc0/build/src/lib.rs#L278

            let prover = default_prover();
            let receipt = prover.prove(env, HELLO_GUEST_ELF).unwrap().receipt;

            receipt.verify(guest_id).unwrap();

            let journal: bool = receipt.journal.decode().unwrap();
            println!("Output from verify: {:?}", journal);
        },
        _ => println!("No arguments provided! Expected execute, prove or verify!"),
    }
}

