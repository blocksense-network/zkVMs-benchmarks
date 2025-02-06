use zkvms_host_io::{PublicInput, PrivateInput, foreach_public_input_field, foreach_private_input_field, read_args, RunType::{Execute, Prove, Verify}, RunWith};
use std::io::{self, Write};
use std::process::{Command, Stdio};
use regex::Regex;

static PUBLIC_INPUT_PATH: &str = "public_input.bin";
static PRIVATE_INPUT_PATH: &str = "private_input.bin";

fn get_with_sizes(flat: &str) -> String {
    let mut values = flat
        .split('[')
        .map(|x| x.trim())
        .skip(1);
    let current = values
        .next()
        .unwrap_or(flat);

    // 1D collection or not a collection
    if current != "" {
        let size = 1 + current
            .clone()
            .to_string()
            .chars()
            .take_while(|x| *x != ']')
            .map(|x| (x == ',') as usize)
            .sum::<usize>();

        (if size > 1 { size.to_string() } else { String::new() })
            + "["
            + current
            + &values
                .map(|x| "[".to_string() + x)
                .collect::<String>()
    }
    // ND collection
    else {
        let size: usize = values
            .clone()
            .count();

        let subcollections = values
            .map(|x| get_with_sizes(x))
            .collect::<String>();

        size.to_string()
            + "["
            + &subcollections
    }
}

macro_rules! build_input {
    ($input:expr , $path:ident , $type:ident) => {
        |run_info: &RunWith| {
            let numreg: Regex = Regex::new("(?:^|[^A-Za-z])([0-9]+)").unwrap();
            let stringreg: Regex = Regex::new("\\\"[^\"]*\\\"").unwrap();

            let mut ret: Vec<u64> = Vec::new();
            $type!{
                let flat = format!("{:?}", $input.yield)
                    .replace("false", "0")
                    .replace("true",  "1")
                    .replace('(', "[")
                    .replace(')', "]")
                    .replace('{', "[")
                    .replace('{', "]");

                let flat = get_with_sizes(&flat);

                let numbers = numreg
                    .captures_iter(&flat)
                    .map(|cap|
                        cap.get(1)
                            .unwrap()
                            .as_str()
                            .to_string()
                            .parse::<u64>()
                            .unwrap())
                    .collect::<Vec<u64>>();

                ret.extend(numbers);

                // let strings: Vec<&str> = stringreg
                //     .captures_iter(&flat)
                //     .map(|cap| cap.get(0).unwrap().as_str())
                //     .collect();
                //
                // panic!("{:#?}", strings);
            }
            let bytes = ret
                .iter()
                .map(|x| x.to_be_bytes())
                .flatten()
                .collect::<Vec<u8>>();
            std::fs::write($path, bytes);
            format!("{}:file", $path)
        }
    };
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

    let public_input = build_input!(
        run_info.public_input,
        PUBLIC_INPUT_PATH,
        foreach_public_input_field)(&run_info);

    let private_input = build_input!(
        run_info.private_input,
        PRIVATE_INPUT_PATH,
        foreach_private_input_field)(&run_info);

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
                .arg("--public").arg(public_input)
                .arg("--private").arg(private_input)
                .arg("--output").arg(output));
        },
        Prove => {
            run(zkwasm_command("prove")
                .arg("--public").arg(public_input)
                .arg("--private").arg(private_input)
                .arg("--output").arg(output));
        },
        Verify => {
            run(zkwasm_command("prove")
                .arg("--public").arg(public_input)
                .arg("--private").arg(private_input)
                .arg("--output").arg(output.clone()));

            run(Command::new("zkwasm-cli")
                .arg("--params").arg(params)
                .arg("prog").arg("verify")
                .arg("--output").arg(output));
        },
    }
}
