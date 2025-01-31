use zkvms_host_io::{Input, foreach_input_field, read_args, RunType::{Execute, Prove, Verify}};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use regex::Regex;

fn build_input(input: &Input) -> String {
    let numreg: Regex = Regex::new("(?:^|[^A-Za-z])([0-9]+)").unwrap();

    let mut ret = String::new();
    foreach_input_field!{
        let flat = format!("{:?}", input.yield)
            .replace("false", "0")
            .replace("true",  "1");

        let numbers: Vec<&str> = numreg
            .captures_iter(&flat)
            .map(|cap| cap.get(1).unwrap().as_str())
            .collect();

        for num in numbers {
            ret.push_str(num);
            ret.push_str(":i64,");
        }
    }
    ret.pop(); // removes trailing comma
    ret
}

fn zkwasm_command(subcmd: &str) -> Command {
    let mut command = Command::new("zkwasm-cli");
    command
        .arg("--params").arg("./params")
        .arg("prog").arg(subcmd)
        .arg("--wasm").arg(env!("GUEST_PATH"));
    command
}

fn run(cmd: &mut Command) {
    assert!(cmd.status().expect("couldn't execute command!").success());
}

fn main() {
    let run_info = read_args();

    let k = run_info
        .env_or(
            "ZKWASM_K",
            "19",
        );

    let scheme = run_info
        .env_or(
            "ZKWASM_SCHEME",
            "shplonk",
        );

    run(zkwasm_command("setup")
        .arg("-k").arg(k)
        .arg("--scheme").arg(scheme));

    let input = build_input(&run_info.input);

    let output = run_info
        .env_or(
            "ZKWASM_OUTPUT",
            "./output",
        );

    let params = run_info
        .env_or(
            "ZKWASM_PARAMS",
            "./params",
        );

    match run_info.run_type {
        Execute => {
            run(zkwasm_command("dry-run")
                .arg("--private").arg(input)
                .arg("--output").arg(output));
        },
        Prove => {
            run(zkwasm_command("prove")
                .arg("--private").arg(input)
                .arg("--output").arg(output));
        },
        Verify => {
            run(zkwasm_command("prove")
                .arg("--private").arg(input)
                .arg("--output").arg(output.clone()));

            run(Command::new("zkwasm-cli")
                .arg("--params").arg(params)
                .arg("prog").arg("verify")
                .arg("--output").arg(output));
        },
    }
}
