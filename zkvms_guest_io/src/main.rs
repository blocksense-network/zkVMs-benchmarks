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

    let guests: Vec<&str> = env!("PROGRAMS")
        .split(',')
        .filter(|x| !x.is_empty())
        .collect();
    let ignored = cli.ignore.unwrap_or(Vec::new());

    for guest in guests.into_iter() {
        if ignored.iter().any(|i| guest.contains(i)) {
            continue;
        }

        println!("== Executing {} ==", guest);

        let output = Command::new(guest)
            .args(cli.zkvm_args.clone())
            .stdout(Stdio::piped())
            .output();

        if let Err(msg) = output {
            println!("Failed to run command {}!", guest);
            println!("{msg}");
            if cli.fail_propagation {
                break;
            }
            continue;
        }
        let output = output.unwrap();

        if !output.stdout.is_empty() {
            print!("{}", String::from_utf8(output.stdout).unwrap());
        }
        if !output.stderr.is_empty() {
            print!("{}", String::from_utf8(output.stderr).unwrap());
        }

        if cli.fail_propagation && !output.status.success() {
            break;
        }
    }
}
