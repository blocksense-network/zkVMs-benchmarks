use sp1_sdk::{EnvProver, ProverClient, SP1ProofWithPublicValues, SP1Stdin, SP1VerifyingKey};
use zkvms_host_io::{
    benchmarkable, foreach_input_field, output_proof_size, read_args, Input,
    RunType::{Execute, Prove, Verify},
};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_bytes!("./guest");

fn build_stdin(input: &Input) -> SP1Stdin {
    let mut stdin = SP1Stdin::new();
    foreach_input_field! {
        stdin.write(&input.yield);
    }
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
        Execute => benchmarkable! {
            let (output, report) = client.execute(FIBONACCI_ELF, &stdin).run().unwrap();

            println!("Program executed successfully.");
            println!("{:?}", output);
            println!("Number of cycles: {}", report.total_instruction_count());
        },
        Prove => benchmarkable! {
            let (proof, _) = prove(&client, stdin.clone());

            output_proof_size(&proof);

            println!("Successfully generated proof!");
        },
        Verify => {
            let (proof, vk) = prove(&client, stdin.clone());
            println!("Successfully generated proof!");

            output_proof_size(&proof);

            benchmarkable! {
                client.verify(&proof, &vk).expect("failed to verify proof");
                println!("Successfully verified proof!");
            }
        }
    }
}
