use nexus_sdk::{stwo::seq::Stwo, Local, Prover, Verifiable, Viewable};
use zkvms_host_io::{
    benchmarkable, output_proof_size, read_args, Input, Return,
    RunType::{Execute, Prove, Verify},
};

fn main() {
    let run_info = read_args();
    if run_info.run_type == Execute {
        panic!("Execution is not supported!");
    }

    let elf_path = std::env::var("ELF_PATH").expect("ELF PATH is missing");

    match run_info.run_type {
        Execute => unreachable!(),
        Prove => benchmarkable! {
            // Stwo<T> doesn't derive Clone
            println!("Loading guest...");
            let prover: Stwo<Local> = Stwo::new_from_file(&elf_path).expect("failed to load guest program");

            println!("Proving execution of vm...");
            let (view, proof) = prover
                .prove_with_input(&run_info.private_input, &run_info.public_input)
                .expect("failed to prove program");

            output_proof_size(&proof);

            println!(
                " output is {:?}!",
                view
                .public_output::<Return>()
                .expect("failed to deserialize output")
                );

            println!(">>>>> Logging\n{}<<<<<", view.logs().expect("failed to retrieve debug logs").join(""));
        },
        Verify => {
            // Stwo<T> doesn't derive Clone
            println!("Loading guest...");
            let prover: Stwo<Local> =
                Stwo::new_from_file(&elf_path).expect("failed to load guest program");

            println!("Proving execution of vm...");
            let (view, proof) = prover
                .prove_with_input(&run_info.private_input, &run_info.public_input)
                .expect("failed to prove program");

            output_proof_size(&proof);

            println!(
                " output is {:?}!",
                view.public_output::<Return>()
                    .expect("failed to deserialize output")
            );

            println!(
                ">>>>> Logging\n{}<<<<<",
                view.logs().expect("failed to retrieve debug logs").join("")
            );

            benchmarkable! {
                print!("Verifying execution...");
                proof.verify(&view).expect("failed to verify proof");
                println!("  Succeeded!");
            }
        }
    }
}
