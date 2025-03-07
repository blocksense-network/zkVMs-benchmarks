use clap::Parser;
use std::process::{Command, Stdio};

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

    /// Arguments which are passed to each tool for a single guest and single zkVM
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    zkvm_args: Vec<String>,
}

fn main() {
    let cli = Cli::parse();

    let zkvm_guest_commands: Vec<&str> = env!("PROGRAMS")
        .split(',')
        .filter(|x| !x.is_empty())
        .collect();
    let ignored = cli.ignore.unwrap_or(Vec::new());

    for zkvm_guest_command in zkvm_guest_commands.into_iter() {
        if ignored.iter().any(|i| zkvm_guest_command.contains(i)) {
            continue;
        }

        println!("== Executing {} ==", zkvm_guest_command);

        let output = Command::new(zkvm_guest_command)
            .args(cli.zkvm_args.clone())
            .stdout(Stdio::piped())
            .output();

        if let Err(msg) = output {
            println!("Failed to run command {}!", zkvm_guest_command);
            println!("{msg}");
            if cli.fail_propagation {
                break;
            }
            continue;
        }
        // The if above makes sure this is an Ok
        let output = output.unwrap();

        if !output.stdout.is_empty() {
            print!(
                "{}",
                String::from_utf8(output.stdout).expect("failed to convert stdout to String")
            );
        }
        if !output.stderr.is_empty() {
            print!(
                "{}",
                String::from_utf8(output.stderr).expect("failed to convert stderr to String")
            );
        }

        if cli.fail_propagation && !output.status.success() {
            break;
        }
    }
}
