use nexus_sdk::{
    compile::CompileOpts,
    nova::seq::{Generate, Nova, PP},
    Local, Prover, Verifiable,
};
use zkvms_host_io::{
    benchmarkable, read_args, Input, Output,
    RunType::{Execute, Prove, Verify},
};

fn main() {
    let run_info = read_args();
    if run_info.run_type == Execute {
        panic!("Execution is not supported!");
    }

    let elf_path = std::env::var("ELF_PATH").expect("ELF PATH is missing");

    println!("Setting up Nova public parameters...");
    let pp: PP = PP::generate().expect("failed to generate parameters");

    match run_info.run_type {
        Execute => unreachable!(),
        Prove => benchmarkable! {
            // Nova<T> doesn't derive Clone
            println!("Loading guest...");
            let prover: Nova<Local> = Nova::new_from_file(&elf_path).expect("failed to load guest program");

            println!("Proving execution of vm...");
            let proof = prover
                .prove_with_input::<Input>(&pp, &run_info.input)
                .expect("failed to prove program");

            println!(
                " output is {:?}!",
                proof
                .output::<Output>()
                .expect("failed to deserialize output")
                );

            println!(">>>>> Logging\n{}<<<<<", proof.logs().join(""));
        },
        Verify => {
            // Nova<T> doesn't derive Clone
            println!("Loading guest...");
            let prover: Nova<Local> =
                Nova::new_from_file(&elf_path).expect("failed to load guest program");

            println!("Proving execution of vm...");
            let proof = prover
                .prove_with_input::<Input>(&pp, &run_info.input)
                .expect("failed to prove program");

            println!(
                " output is {:?}!",
                proof
                    .output::<Output>()
                    .expect("failed to deserialize output")
            );

            println!(">>>>> Logging\n{}<<<<<", proof.logs().join(""));

            benchmarkable! {
                print!("Verifying execution...");
                proof.verify(&pp).expect("failed to verify proof");
            }
        }
    }
}
