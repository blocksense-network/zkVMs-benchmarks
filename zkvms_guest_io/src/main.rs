use clap::Parser;
use std::process::{Command, Stdio};
use json::{object, parse, JsonValue, Null};
use std::io::{Error, Write};
use std::fs::{read_to_string, OpenOptions};
use smbioslib::*;
use sysinfo::System;
use itertools::Itertools;

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

static COMMAND_LOG_PATH: &str = "/tmp/output.log";
static METRICS_TEMP_OUTPUT_PATH: &str = "/tmp/current_metrics";
static PROOF_SIZE_FILE_PATH: &str = "/tmp/proof_size";

fn run_command(zkvm_guest_command: &str, operation: &str) -> Result<std::process::Output, Error> {
    Command::new("runexec")
        .args(["--no-container", "--output", COMMAND_LOG_PATH, "--"])
        .args([zkvm_guest_command, operation])
        .arg("--benchmark")
        .args([ "--metrics-output", METRICS_TEMP_OUTPUT_PATH ])
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
        "benchmarking": [],
        "hardware": {
            cpu: [],
            memory: {
                model: Null,
                size: 0,
                speed: Null,
            },
            hardwareAcceleration: [],
            accelerated: false
        },
    };

    // Always available information
    let sys = System::new_all();

    let cpus = sys.cpus().into_iter().unique_by(|c| c.brand()).collect::<Vec<_>>();
    for cpu in cpus {
        let mut hcpu = JsonValue::new_object();

        hcpu["model"] = cpu.brand().into();
        // This core count will be wrong in case the system has more than one CPUs
        hcpu["cores"] = System::physical_core_count().unwrap_or(0).into();
        hcpu["speed"] = cpu.frequency().into();

        runs["hardware"]["cpu"].push(hcpu);
    }

    runs["hardware"]["memory"]["size"] = sys.total_memory().into();

    // Available with root permissions
    // Note: it is not enough to just run the executable with sudo. runexec connects
    // to DBus, so you'll need a proper root user session.
    // Either login through another TTY as root, or use `machinectl shell root@`
    if let Ok(sys) = table_load_from_device() {
        // Fix CPU core counts
        let cpus = sys.filter(|cpu: &SMBiosProcessorInformation| true).collect::<Vec<SMBiosProcessorInformation>>();
        for mut hcpu in runs["hardware"]["cpu"].members_mut() {
            if let Some(cpu) = cpus.iter().find(|cpu|
                if let Some(ver) = cpu.processor_version().ok() {
                    ver.trim() == hcpu["model"].to_string().trim()
                }
                else {
                    false
                }
            ) {
                if let Some(CoreCount::Count(cores)) = cpu.core_count() {
                    hcpu["cores"] = cores.into();
                }
            }
        }

        // Add memory model and speed
        if let Some(memory) = sys.find_map(|memory: SMBiosMemoryDevice| Some(memory)) {
            if let Some(model) = memory.part_number().ok() {
                runs["hardware"]["memory"]["model"] = model.trim().into();
            }
            if let Some(MemorySpeed::MTs(speed)) = memory.speed() {
                runs["hardware"]["memory"]["speed"] = speed.into();
            }
        }
    }

    'guest_iter: for zkvm_info in zkvm_guest_commands.into_iter() {
        let zkvm_info_fields: Vec<&str> = zkvm_info.split('|').collect();
        let zkvm = zkvm_info_fields[0];

        if ignored.iter().any(|i| zkvm.contains(i)) {
            continue;
        }

        let zkvmRev = zkvm_info_fields[1];
        let guest = zkvm_info_fields[2];
        let commit = zkvm_info_fields[3];
        let zkvm_guest_command = zkvm_info_fields[4];

        let mut run = JsonValue::new_object();
        run["zkvmName"] = zkvm.into();
        run["zkvmRev"] = zkvmRev.into();
        run["programName"] = guest.into();
        run["commit"] = commit.into();

        for operation in ["execute", "prove", "verify"] {
            println!("== {operation} {zkvm} ==");

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
                if let Some(log) = read_to_string(COMMAND_LOG_PATH).ok() {
                    println!("{log}");
                }
                continue;
            }

            let raw_data = &read_to_string(METRICS_TEMP_OUTPUT_PATH)
                .ok()
                .unwrap();
            run[operation] = json::parse(raw_data).unwrap();
            run[operation]["memory"] = get_runexec_value(&stdout, "memory", 'B').parse::<u64>().unwrap().into();

            let proofSize = &read_to_string(PROOF_SIZE_FILE_PATH)
                .ok()
                .unwrap();
            run[operation]["proofSize"] = proofSize.parse::<u64>().unwrap().into();
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
