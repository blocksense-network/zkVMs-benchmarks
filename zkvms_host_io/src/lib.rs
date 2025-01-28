use clap::{Parser, ValueEnum};
use num_traits::NumCast;
use serde::{ Serialize, Deserialize };
use std::fs::read_to_string;
pub use input_macros::foreach_input_field;

static DEFAULT_PUBLIC_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"));

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What the ZKVM is going to do
    run_type: RunType,

    public_input: Option<String>,
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

    let contents: String = if cli.public_input.is_some() {
            read_to_string(cli.public_input.unwrap()).unwrap()
        } else {
            DEFAULT_PUBLIC_INPUT.to_string()
        };
    let val: Input = toml::from_str(&contents).unwrap();

    RunWith {
        run_type: cli.run_type,
        input: val,
    }
}
