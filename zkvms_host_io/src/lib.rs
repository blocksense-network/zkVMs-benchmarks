use clap::{Parser, ValueEnum};
use num_traits::NumCast;
use serde::{ Serialize, Deserialize };
use env_file_reader::read_str;
use std::{env, option::Option, fs::read_to_string, collections::HashMap};
pub use input_macros::foreach_input_field;

static DEFAULT_PUBLIC_INPUT: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"));
static DEFAULT_ENV: &str = include_str!(concat!(env!("INPUTS_DIR"), "/default.env"));

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunWith<T> {
    pub run_type: RunType,
    pub input: T,
    pub default_env: HashMap<String, String>,
}

impl<T> RunWith<T> {
    pub fn env_then_or<U>(&self, variable_name: &str, then_apply: fn(String) -> Option<U>, else_const: U) -> U {
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

input_macros::generate_input_struct!();

pub fn read_args() -> RunWith<Input> {
    let cli = Cli::parse();

    let contents: String = if cli.public_input.is_some() {
            read_to_string(cli.public_input.unwrap()).unwrap()
        } else {
            DEFAULT_PUBLIC_INPUT.to_string()
        };
    let input: Input = toml::from_str(&contents).unwrap();

    let default_env = read_str(DEFAULT_ENV).unwrap();

    RunWith {
        run_type: cli.run_type,
        input,
        default_env,
    }
}
