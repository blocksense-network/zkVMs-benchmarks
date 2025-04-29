use clap::Parser;
use std::process::{Command, Stdio};
use json::{object, parse, JsonValue, Null};

/// A CLI tool for running and benchmarking a guest program inside all
/// supported zkVMs.
/// This binary has been built with a single guest program in mind.
/// If you want to run or benchmark your own guest program inside a zkVM,
/// head on over to https://github.com/blocksense-network/zkVMs-benchmarks
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    /// Ignored zkVMs. Values are substrings of names.
    #[arg(short, long, value_delimiter = ',', num_args = 1..)]
    ignore: Option<Vec<String>>,

    /// Make one failiure stop the entire process
    #[arg(short, long)]
    fail_propagation: bool,
}

fn run_command(zkvm_guest_command: &str, operation: &str) -> Result<std::process::Output, Error> {
    Command::new(zkvm_guest_command)
        .arg(operation)
        .arg("--benchmark")
        .args([ "--metrics-output", "/tmp/current_metrics" ])
        .stdout(Stdio::piped())
        .output()
}

fn main() {
    let cli = Cli::parse();

    // This is set by zkvms_guest_io/default.nix
    let zkvm_guest_commands: Vec<&str> = env!("PROGRAMS")
        .split(',')
        .filter(|x| !x.is_empty())
        .collect();
    let ignored = cli.ignore.unwrap_or(Vec::new());

    let mut runs = object! {
        "benchmarking": []
    };

    'guest_iter: for zkvm_guest_command in zkvm_guest_commands.into_iter() {
        if ignored.iter().any(|i| zkvm_guest_command.contains(i)) {
            continue;
        }

        let mut run = JsonValue::new_object();
        run["name"] = zkvm_guest_command.into();

        for operation in ["execute", "prove", "verify"] {
            println!("== {operation} {zkvm_guest_command} ==");

            let output = run_command(zkvm_guest_command, operation);

            // Couldn't run the command
            if let Err(msg) = output {
                println!("Failed to run command!");
                println!("{msg}");
                if cli.fail_propagation {
                    break 'guest_iter;
                }
                continue;
            }

            // The command ran and therefore produced some output
            let output = output.unwrap();

            // The command ran but exited with non-zero status code
            if !output.status.success() {
                println!("Command failed!");
            }

            let stdout = String::from_utf8(output.stdout).expect("failed to convert stdout to String");
            println!("{stdout}");

            if !output.stderr.is_empty() {
                print!(
                    "{}",
                    String::from_utf8(output.stderr).expect("failed to convert stderr to String")
                );
            }

            // The command ran but exited with non-zero status code
            if !output.status.success() {
                break 'guest_iter;
            }

            let raw_data = &read_to_string("/tmp/current_metrics")
                .ok()
                .unwrap();
            run[operation] = json::parse(raw_data).unwrap();
        }

        runs["benchmarking"].push(run);
    }

    println!("{}", runs.dump());
}
