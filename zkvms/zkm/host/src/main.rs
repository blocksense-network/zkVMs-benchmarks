use anyhow::{bail, Result};
use std::{env, fs::read, time::Instant};
use zkm_sdk::{
    prover::ClientCfg,
    prover::{ProverInput, ProverResult},
    ProverClient,
};

use zkvms_host_io::{
    benchmarkable, read_args,
    RunType::{Execute, Prove, Verify},
};

async fn setup(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
    client_cfg: &mut ClientCfg,
) {
    let generation = prover_client
        .setup_and_generate_sol_verifier(&client_cfg.zkm_prover, &client_cfg.vk_path, &prover_input)
        .await;

    if let Err(e) = generation {
        panic!("Failed setup! Error: {e}");
    }
}

async fn get_proof(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
) -> ProverResult {
    let proving_result = prover_client.prover.prove(&prover_input, None).await;

    if let Ok(Some(prover_result)) = proving_result {
        prover_result
    } else {
        panic!("Failed to generate proof!");
    }
}

async fn execute(prover_client: &mut ProverClient, prover_input: &mut ProverInput) {
    let prover_result = get_proof(prover_client, prover_input).await;

    prover_client
        .print_guest_execution_output(true, &prover_result)
        .expect("print guest program excution's output false.")
}

async fn prove(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
    vk_path: &String,
    proof_results_path: &String,
) {
    let prover_result = get_proof(prover_client, prover_input).await;

    prover_client
        .process_proof_results(&prover_result, &prover_input, &proof_results_path, "local")
        .expect("process proof results error");
}

#[tokio::main]
async fn main() -> Result<()> {
    let run_info = read_args();
    if run_info.run_type == Verify {
        panic!("Off-chain verification is not supported!");
    }

    let seg_size: u32 = run_info.env_then_or("SEG_SIZE", |seg| seg.parse::<u32>().ok(), 65536);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missing");

    let proof_results_path = run_info.env_or("PROOF_RESULTS_PATH", "/tmp/contracts");
    let vk_path = run_info.env_or("VERIFYING_KEY_PATH", "/tmp/input");

    let mut client_config = ClientCfg::new("local".to_string(), vk_path.to_owned());

    let mut prover_client = ProverClient::new(&client_config).await;

    // Set input
    let mut public_inputstream = Vec::new();
    bincode::serialize_into(&mut public_inputstream, &run_info.public_input)
        .expect("public_input serialization failed");

    let mut private_inputstream = Vec::new();
    bincode::serialize_into(&mut private_inputstream, &run_info.private_input)
        .expect("private_input serialization failed");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        execute_only: run_info.run_type == Execute,
        seg_size,
        public_inputstream,
        private_inputstream,
        ..Default::default()
    };

    let start = Instant::now();
    setup(&mut prover_client, &mut prover_input, &mut client_config).await;

    match run_info.run_type {
        // only excute the guest program without generating the proof.
        Execute => benchmarkable! {
            execute(&mut prover_client, &mut prover_input).await;
        },
        // excute the guest program and generate the proof
        Prove => benchmarkable! {
            prove(&mut prover_client, &mut prover_input, &vk_path, &proof_results_path).await;
        },
        Verify => unreachable!(),
    }
    let end = Instant::now();
    let elapsed = end.duration_since(start);
    println!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
