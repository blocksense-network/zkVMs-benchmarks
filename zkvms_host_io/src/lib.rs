use chrono::Local;
use clap::{Parser, ValueEnum};
use env_file_reader::read_str;
pub use input_macros::{
    benchmarkable, foreach_input_field, foreach_private_input_field, foreach_public_input_field,
};
use json::{object, parse, JsonValue};
use num_traits::NumCast;
use serde::{Deserialize, Serialize};
use std::{
    collections::*,
    env,
    fs::{read_to_string, write, OpenOptions},
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};
use sysinfo::System;

static DEFAULT_PUBLIC_INPUT: &str =
    include_str!(concat!(env!("INPUTS_DIR"), "/default_public_input.toml"));
static DEFAULT_PRIVATE_INPUT: &str =
    include_str!(concat!(env!("INPUTS_DIR"), "/default_private_input.toml"));
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
    pub fn env_then_or<T>(
        &self,
        variable_name: &str,
        then_apply: fn(String) -> Option<T>,
        else_const: T,
    ) -> T {
        env::var(variable_name).ok().and_then(then_apply).unwrap_or(
            self.default_env
                .get(variable_name)
                .and_then(|x| then_apply(x.clone()))
                .unwrap_or(else_const),
        )
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

/// Used by the "benchmarkable" macro. Takes run_info and two vectors of start and
/// end instants for each benchmarkiteration.
pub fn emit_benchmark_results(run_info: RunWith, starts: Vec<Instant>, ends: Vec<Instant>) {
    let now = Local::now();

    let mut run = object! {
        "zkvmName": "",
        "programName": "",
        "operation": "",
        "timeStarted": "",
        "timeEnded": "",
        "repeats": 0,
        "duration": 0,
        "memory": 0,
        "input": "",
        "proofSize": 0
    };

    run["zkvmName"] = env!("ZKVM").into();
    run["programName"] = env!("GUEST").into();
    run["operation"] = match run_info.run_type {
        RunType::Execute => "execute",
        RunType::Prove => "prove",
        RunType::Verify => "verify",
    }
    .into();

    let duration = *ends.last().unwrap() - *starts.first().unwrap();
    run["timeStarted"] = (now - duration).to_string().into();
    run["timeEnded"] = now.to_string().into();
    run["repeats"] = run_info.repeats.into();

    let durations = starts
        .iter()
        .zip(ends.iter())
        .map(|(&s, &e)| e - s)
        .collect::<Vec<Duration>>();
    let average = durations.iter().sum::<Duration>() / durations.len() as u32;
    run["duration"] = average.as_secs_f32().into();

    let mut data = if run_info.append
        && run_info.output_file.is_some()
        && Path::new(run_info.output_file.as_ref().unwrap()).exists()
    {
        let mut raw_data = &read_to_string(run_info.output_file.as_ref().unwrap())
            .ok()
            .unwrap();
        parse(raw_data).expect("file is not a JSON")
    } else {
        object! {
            benchmarking: {
                runs: [],
                commit: ""
            }
        }
    };

    if !data.has_key("hardware") {
        let sys = System::new_all();

        let mut cpu = JsonValue::new_object();
        cpu["model"] = sys.cpus()[0].brand().into();
        cpu["cores"] = System::physical_core_count()
            .unwrap_or(sys.cpus().len() / 2)
            .into();
        cpu["speed"] = (sys.cpus()[0].frequency() as f32 / 1000.0).into(); // in GHz

        let mut memory = JsonValue::new_object();
        memory["model"] = "".into();
        memory["size"] = (sys.total_memory() as f32 / 1000000000.0).into(); // in GB
        memory["speed"] = 0.into();

        let mut accel = JsonValue::new_object();
        accel["used"] = false.into();
        accel["model"] = "".into();
        accel["cores"] = 0.into();
        accel["speed"] = 0.into();

        data["benchmarking"]["hardware"] = JsonValue::new_object();
        data["benchmarking"]["hardware"]["cpu"] = cpu;
        data["benchmarking"]["hardware"]["memory"] = memory;
        data["benchmarking"]["hardware"]["hardwareAcceleration"] = accel;
    }

    data["benchmarking"]["runs"].push(run);

    if let Some(path) = run_info.output_file {
        write(path, data.dump())
            .ok()
            .expect("Couldn't output to file");
    } else {
        println!("{}", data.dump());
    }
}
