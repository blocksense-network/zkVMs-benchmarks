use zkvms_host_io::{read_args, RunType::{ Execute, Prove, Verify }};
use sp1_sdk::{ProverClient, EnvProver, SP1Stdin, SP1ProofWithPublicValues, SP1VerifyingKey};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_bytes!("./guest");

type Input = (Vec<Vec<bool>>, u32, Vec<Vec<u32>>);

fn build_stdin((graph, colors, coloring): &Input) -> SP1Stdin {
    let mut stdin = SP1Stdin::new();
    stdin.write(&graph);
    stdin.write(&colors);
    stdin.write(&coloring);
    stdin
}

fn prove(client: &EnvProver, stdin: SP1Stdin) -> (SP1ProofWithPublicValues, SP1VerifyingKey) {
    let (pk, vk) = client.setup(FIBONACCI_ELF);
    let proof = client
        .prove(&pk, &stdin)
        .run()
        .expect("failed to generate proof");
    (proof, vk)
}

fn main() {
    let run_info = read_args();
    let stdin = build_stdin(&run_info.input);

    sp1_sdk::utils::setup_logger();
    let client = ProverClient::new();

    match run_info.run_type {
        Execute => {
            let (output, report) = client.execute(FIBONACCI_ELF, &stdin).run().unwrap();

            println!("Program executed successfully.");
            println!("{:?}", output);
            println!("Number of cycles: {}", report.total_instruction_count());
        },
        Prove => {
            let _ = prove(&client, stdin);
            println!("Successfully generated proof!");
        },
        Verify => {
            let (proof, vk) = prove(&client, stdin);
            println!("Successfully generated proof!");

            client.verify(&proof, &vk).expect("failed to verify proof");
            println!("Successfully verified proof!");
        },
    }
}
