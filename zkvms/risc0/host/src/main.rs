use risc0_zkvm::{default_prover, default_executor, ExecutorEnv};

static HELLO_GUEST_ELF: &[u8] = include_bytes!("../../../../zkvms/risc0/guest/target/riscv32im-risc0-zkvm-elf/release/guest");

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
            let receipt = prover.prove(env, HELLO_GUEST_ELF).unwrap().receipt;

            // Extract journal of receipt
            let journal: bool = receipt.journal.decode().unwrap();
            // Print, notice, after committing to a journal, the private input became public
            println!("Output from journal: {:?}", journal);
        },
        "verify" => {
            let prover = default_prover();
            let receipt = prover.prove(env, HELLO_GUEST_ELF).unwrap().receipt;

            // receipt.verify(HELLO_GUEST_ID).unwrap();

            // let journal: bool = receipt.journal.decode().unwrap();
            // println!("Output from verify: {:?}", journal);
        },
        _ => println!("No arguments provided! Expected execute, prove or verify!"),
    }
}

