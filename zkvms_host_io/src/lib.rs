use clap::{Parser, ValueEnum};
use num_traits::NumCast;
use serde::{ Serialize, Deserialize };
pub use input_macros::foreach_input_field;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What the ZKVM is going to do
    run_type: RunType,

    #[arg(default_value = "./public_input.toml")]
    public_input: String,
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

input_macros::generate_input_struct!();

pub fn read_args() -> RunWith<Input> {
    let cli = Cli::parse();

    let val: Input = toml::from_str(&std::fs::read_to_string(cli.public_input).unwrap()).unwrap();

    RunWith {
        run_type: cli.run_type,
        input: val,
    }
}
