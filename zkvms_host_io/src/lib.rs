use clap::{Parser, ValueEnum};
use num_traits::NumCast;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What the ZKVM is going to do
    run_type: RunType,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RunType {
    Execute,
    Prove,
    Verify,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RunWith<T> {
    pub run_type: RunType,
    pub input: T,
}

fn read_input() -> (Vec<Vec<bool>>, u32, Vec<Vec<u32>>) {
    include!(env!("INPUTS"))
}

pub fn read_args() -> RunWith<(Vec<Vec<bool>>, u32, Vec<Vec<u32>>)> {
    let cli = Cli::parse();

    RunWith {
        run_type: cli.run_type,
        input: read_input(),
    }
}
