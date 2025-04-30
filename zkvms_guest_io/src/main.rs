use clap::Parser;
use std::process::{Command, Stdio};
use json::{object, parse, JsonValue, Null};
use std::io::{Error, Write};
use std::fs::{read_to_string, OpenOptions};

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

    /// Put the resultant output into a file of the given path
    #[arg(short = 'o', long)]
    metrics_output: Option<String>,

    /// Append the resultant output to the given file, instead of replacing it
    #[arg(short, long)]
    append: bool,
}

fn run_command(zkvm_guest_command: &str, operation: &str) -> Result<std::process::Output, Error> {
    Command::new("runexec")
        .args(["--no-container", "--"])
        .args([zkvm_guest_command, operation])
        .arg("--benchmark")
        .args([ "--metrics-output", "/tmp/current_metrics" ])
        .stdout(Stdio::piped())
        .output()
}

fn get_runexec_value(output: &String, name: &str, end: char) -> String {
    let start_bytes = output.find(name).unwrap();
    let right_half = &output[start_bytes + name.len() + 1..];
    right_half[..right_half.find(end).unwrap()].to_string()
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

            // Couldn't run runexec
            if let Err(msg) = output {
                println!("Failed to run command!");
                println!("{msg}");
                if cli.fail_propagation {
                    break 'guest_iter;
                }
                continue;
            }

            // runexec ran and therefore produced some output
            let output = output.unwrap();

            // runexec exited with non-zero status code
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

            // runexec ran but exited with non-zero status code
            if !output.status.success() {
                break 'guest_iter;
            }

            // The guest program ran but exited with non-zero status code
            if get_runexec_value(&stdout, "returnvalue", '\n') != "0" {
                run[operation] = Null;
                continue;
            }

            let raw_data = &read_to_string("/tmp/current_metrics")
                .ok()
                .unwrap();
            run[operation] = json::parse(raw_data).unwrap();
            run[operation]["memory"] = get_runexec_value(&stdout, "memory", 'B').parse::<u64>().unwrap().into();
        }

        runs["benchmarking"].push(run);
    }

    if let Some(path) = cli.metrics_output {
        let mut outfile = match OpenOptions::new()
            .write(true)
            .create(true)
            .append(cli.append)
            .truncate(!cli.append)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) => {
                panic!("Failed to open metrics output file \"{path}\": {e}");
            }
        };

        if let Err(e) = writeln!(outfile, "{}", runs.dump()) {
            panic!("Failed to write to metrics output file \"{path}\": {e}");
        }
    } else {
        println!("{}", runs.dump());
    }
}
