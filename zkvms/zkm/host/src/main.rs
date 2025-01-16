use anyhow::bail;
use anyhow::Result;
use std::env;
use std::fs::read;
use std::time::Instant;
use zkm_sdk::{prover::ClientCfg, prover::{ProverInput, ProverResult} , ProverClient};

use zkvms_host_io::{read_args, RunType::{ Execute, Prove, Verify }};

async fn get_proof(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
) -> ProverResult {
    let proving_result = prover_client.prover.prove(&prover_input, None).await;

    if let Ok(Some(prover_result)) = proving_result {
        prover_result
    }
    else {
        panic!("Failed to generate proof!");
    }
}

async fn execute(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
) {
    prover_input.execute_only = true;

    let prover_result = get_proof(prover_client, prover_input).await;

    prover_client
        .print_guest_execution_output(false, &prover_result)
        .expect("print guest program excution's output false.")
}

async fn prove(
    prover_client: &mut ProverClient,
    prover_input: &mut ProverInput,
    vk_path: &String,
    proof_results_path: &String,
) {
    prover_input.execute_only = false;

    match prover_client
        .setup_and_generate_sol_verifier("local", &vk_path, &prover_input)
        .await
    {
        Ok(()) => println!("Succussfully setup_and_generate_sol_verifier."),
        Err(e) => panic!("Error during setup_and_generate_sol_verifier: {}", e),
    }

    let prover_result = get_proof(prover_client, prover_input).await;

    prover_client
        .process_proof_results(
            &prover_result,
            &prover_input,
            &proof_results_path,
            "local",
        )
        .expect("process proof results error");
}

#[tokio::main]
async fn main() -> Result<()> {
    let run_info = read_args();
    if run_info.run_type == Verify {
        panic!("Off-chain verification is not supported!");
    }

    let seg_size = env::var("SEG_SIZE")
        .ok()
        .and_then(|seg| seg.parse::<u32>().ok())
        .unwrap_or(8192);

    let elf_path = env::var("ELF_PATH").expect("ELF PATH is missing");

    let proof_results_path = env::var("PROOF_RESULTS_PATH").unwrap_or("/tmp/contracts".to_string());
    let vk_path = env::var("VERIFYING_KEY_PATH").unwrap_or("/tmp/input".to_string());

    let mut client_config: ClientCfg =
        ClientCfg::new("local".to_string(), vk_path.to_owned());

    let mut prover_client = ProverClient::new(&client_config).await;

    // Set input
    let mut public_inputstream = Vec::new();
    bincode::serialize_into(&mut public_inputstream, &run_info.input)
        .expect("private_input serialization failed");

    let mut prover_input = ProverInput {
        elf: read(elf_path).unwrap(),
        seg_size,
        public_inputstream,
        ..Default::default()
    };

    let start = Instant::now();
    match run_info.run_type {
        // only excute the guest program without generating the proof.
        Execute => execute(&mut prover_client, &mut prover_input).await,
        // excute the guest program and generate the proof
        Prove => prove(&mut prover_client, &mut prover_input, &vk_path, &proof_results_path).await,
        Verify => unreachable!(),
    }
    let end = Instant::now();
    let elapsed = end.duration_since(start);
    println!("Elapsed time: {:?} secs", elapsed.as_secs());
    Ok(())
}
