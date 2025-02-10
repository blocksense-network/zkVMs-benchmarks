use clap::{Parser, ValueEnum};
use num_traits::NumCast;
use serde::{ Serialize, Deserialize };
use env_file_reader::read_str;
use std::{env, option::Option, fs::read_to_string, collections::*};
pub use input_macros::{ foreach_input_field, foreach_public_input_field, foreach_private_input_field, benchmarkable };

static DEFAULT_PUBLIC_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"));
static DEFAULT_PRIVATE_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_private_input.toml"));
static DEFAULT_ENV: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default.env"));

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// What the ZKVM is going to do
    run_type: RunType,

    private_input: Option<String>,

    public_input: Option<String>,

    #[arg(short, long)]
    benchmark: bool,

    #[arg(short, long, requires = "benchmark")]
    repeat: Option<usize>,

    #[arg(short, long, requires = "benchmark")]
    millis: bool,

    #[arg(short, long, requires = "benchmark")]
    metrics_output: Option<String>,
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

    pub input: Input,
    pub public_input: PublicInput,
    pub private_input: PrivateInput,

    pub default_env: HashMap<String, String>,
}

impl RunWith {
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

        input,
        public_input,
        private_input,

        default_env,
    }
}
