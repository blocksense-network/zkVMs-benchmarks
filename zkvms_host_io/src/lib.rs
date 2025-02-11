use clap::{Parser, ValueEnum};
use num_traits::NumCast;
use serde::{ Serialize, Deserialize };
use env_file_reader::read_str;
use std::{env, option::Option, fs::read_to_string, collections::*};
pub use input_macros::{ foreach_input_field, foreach_public_input_field, foreach_private_input_field, benchmarkable };

static DEFAULT_PUBLIC_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"));
static DEFAULT_PRIVATE_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_private_input.toml"));
static DEFAULT_ENV: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default.env"));

/// A CLI tool for running and benchmarking guest programs inside a zkVM
/// environment.
/// This binary has been built with a single zkVM and guest program in mind.
/// If you want to run or benchmark your own guest program inside a zkVM,
/// head on over to https://github.com/blocksense-network/zkVMs-benchmarks
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Cli {
    /// What should the zkVM do with the guest
    run_type: RunType,

    /// Path to private input file (in TOML format)
    private_input: Option<String>,

    /// Path to public input file (in TOML format)
    public_input: Option<String>,

    /// Enable benchmark timer and formatted output
    #[arg(short, long)]
    benchmark: bool,

    /// Benchmark the given action multiple times
    #[arg(short, long, requires = "benchmark")]
    repeat: Option<usize>,

    /// Output timings as milliseconds instead of seconds
    #[arg(short, long, requires = "benchmark")]
    millis: bool,

    /// Put the benchmark's formatted output into a file of the given path
    #[arg(short = 'o', long, requires = "benchmark")]
    metrics_output: Option<String>,

    /// Append the benchmark formatted output to the given file, instead of replacing it
    #[arg(short, long, requires = "benchmark")]
    append: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RunType {
    Execute,
    Prove,
    Verify,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunWith {
    pub run_type: RunType,
    pub benchmarking: bool,
    pub repeats: usize,
    pub millis: bool,
    pub output_file: Option<String>,
    pub append: bool,

    pub input: Input,
    pub public_input: PublicInput,
    pub private_input: PrivateInput,

    pub default_env: HashMap<String, String>,
}

impl RunWith {
    /// Returns a value of the given name from the environment,
    /// default environment or the given constant, depending on which
    /// one exists.
    ///
    /// If the variable is from either environments, the `then_apply`
    /// function is executed to convert the String value.
    ///
    /// The default environment is taken from the guest's `default.env`
    pub fn env_then_or<T>(&self, variable_name: &str, then_apply: fn(String) -> Option<T>, else_const: T) -> T {
        env::var(variable_name)
            .ok()
            .and_then(then_apply)
            .unwrap_or(self
                .default_env
                .get(variable_name)
                .and_then(|x| then_apply(x.clone()))
                .unwrap_or(else_const))
    }

    /// Returns a value of the given name from the environment,
    /// default environment or the given constant, depending on which
    /// one exists.
    ///
    /// The default environment is taken from the guest's `default.env`
    pub fn env_or(&self, variable_name: &str, else_const: &str) -> String {
        self.env_then_or(variable_name, |x| Some(x), else_const.to_string())
    }
}

input_macros::generate_output_type_input_struct!();

pub fn read_args() -> RunWith {
    let cli = Cli::parse();

    let public_contents: String = if cli.public_input.is_some() {
            read_to_string(cli.public_input.unwrap()).unwrap()
        } else {
            DEFAULT_PUBLIC_INPUT.to_string()
        };
    let private_contents: String = if cli.private_input.is_some() {
            read_to_string(cli.private_input.unwrap()).unwrap()
        } else {
            DEFAULT_PRIVATE_INPUT.to_string()
        };
    let input: Input = toml::from_str(&(public_contents.clone() + &private_contents)).unwrap();
    let public_input: PublicInput = toml::from_str(&public_contents).unwrap();
    let private_input: PrivateInput = toml::from_str(&private_contents).unwrap();

    let default_env = read_str(DEFAULT_ENV).unwrap();

    RunWith {
        run_type: cli.run_type,
        benchmarking: cli.benchmark,
        repeats: cli.repeat.unwrap_or(1),
        millis: cli.millis,
        output_file: cli.metrics_output,
        append: cli.append,

        input,
        public_input,
        private_input,

        default_env,
    }
}
