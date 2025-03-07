use clap::Parser;
use std::process::{Command, Stdio};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

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
    let mut threads = Vec::new();
    let ignored = cli.ignore.unwrap_or(Vec::new());
    let fail = Arc::new(AtomicBool::new(false));

    for guest in guests.into_iter() {
        if ignored.iter().any(|i| guest.contains(i)) {
            continue;
        }

        let args = cli.zkvm_args.clone();
        let fail = fail.clone();
        threads.push(
            thread::Builder::new()
                .name(format!(r#"Running "{}""#, guest))
                .spawn(move || {
                    let output = Command::new(guest)
                        .args(args)
                        .stdout(Stdio::piped())
                        .output()
                        .expect("failed to run command");

                    let mut stdout = String::from_utf8(output.stdout).unwrap();
                    if !output.stderr.is_empty() {
                        stdout.push('\n');
                        stdout += &String::from_utf8(output.stderr).unwrap();
                    }

                    print!("== Executing {} ==\n{}", guest, stdout);
                    if !output.status.success() {
                        // Make sure we print a message before failing
                        // There could be a race condition, where we fail, then while
                        // panic is doing it's thing, the main thread exits.
                        println!("Program didn't exist successfully!");
                        fail.store(true, Ordering::Relaxed);
                        panic!();
                    }
                })
                .expect("failed to spawn thread"),
        );
    }

    while threads.iter().any(|t| !t.is_finished())
        && (!cli.fail_propagation || !fail.load(Ordering::Relaxed))
    {
        thread::sleep(Duration::from_millis(200));
    }
}
