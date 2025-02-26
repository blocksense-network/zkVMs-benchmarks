use std::io::{self, Write};
use std::process::{Command, Stdio};
use zkvms_host_io::{
    benchmarkable, foreach_private_input_field, foreach_public_input_field, read_args,
    PrivateInput, PublicInput,
    RunType::{Execute, Prove, Verify},
    RunWith,
};

static PUBLIC_INPUT_PATH: &str = "public_input.bin";
static PRIVATE_INPUT_PATH: &str = "private_input.bin";

/// Inserts array sizes before every square bracket
///
/// # Example
///
/// If `flat` is "[[0,1], [2,3,4], []]"
/// Output will be "3[2[0,1], 3[2,3,4], 0[]]"
fn get_with_sizes(flat: &str) -> String {
    let mut values = flat.split('[').map(|x| x.trim()).skip(1);
    let current = values.next().unwrap_or(flat);

    // 1D collection or not a collection
    if current != "" {
        let size = 1 + current
            .clone()
            .to_string()
            .chars()
            .take_while(|x| *x != ']')
            .map(|x| (x == ',') as usize)
            .sum::<usize>();

        (if size > 1 {
            size.to_string()
        } else {
            String::new()
        }) + "["
            + current
            + &values.map(|x| "[".to_string() + x).collect::<String>()
    }
    // ND collection
    else {
        let size: usize = values.clone().count();

        let subcollections = values.map(|x| get_with_sizes(x)).collect::<String>();

        size.to_string() + "[" + &subcollections
    }
}

/// Creates an anonymous function which takes `run_info`, "serializes" the
/// specified input, outputs it into a file and returns a "path:<PATH>"
/// argument, ready to be passed to zkWasm.
///
/// The macro takes three arguments: run_info input expression, path for file
/// output and the name of a foreach macro.
///
/// For collection types, first the size is emitted and afterwards its actual
/// values.
macro_rules! build_input {
    ($input:expr , $path:ident , $type:ident) => {
        |run_info: &RunWith| {
            let mut ret: Vec<u64> = Vec::new();
            $type! {
                // Simplify input string
                let flat = format!("{:?}", $input.yield)
                    .replace("false", "0")
                    .replace("true",  "1")
                    .replace('(', "[")
                    .replace(')', "]")
                    .replace('{', "[")
                    .replace('}', "]");

                let flat = get_with_sizes(&flat);

                let values = flat
                    .replace('[', ",")
                    .replace(']', " ")
                    .replace(':', ",")
                    .split(',')
                    .map(|val| {
                        let val = val.trim();
                        if let Some(num) = val.parse::<u64>().ok() {
                            vec![num]
                        }
                        else {
                            let val = val.trim_matches('"');
                            let mut size = vec![val.len() as u64];
                            size.extend(val
                                .bytes()
                                .into_iter()
                                .map(|x| x as u64)
                                .collect::<Vec<u64>>());
                            size
                        }
                    })
                    .flatten()
                    .collect::<Vec<u64>>();

                ret.extend(values);
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
        .arg("--params")
        .arg("./params")
        .arg("prog")
        .arg(subcmd)
        .arg("--wasm")
        .arg(env!("GUEST_PATH"));
    command
}

fn run(cmd: &mut Command) {
    assert!(cmd.status().expect("couldn't execute command!").success());
}

fn main() {
    let run_info = read_args();

    let k = run_info.env_or("ZKWASM_K", "19");

    let scheme = run_info.env_or("ZKWASM_SCHEME", "shplonk");

    run(zkwasm_command("setup")
        .arg("-k")
        .arg(k)
        .arg("--scheme")
        .arg(scheme));

    let public_input = build_input!(
        run_info.public_input,
        PUBLIC_INPUT_PATH,
        foreach_public_input_field
    )(&run_info);

    let private_input = build_input!(
        run_info.private_input,
        PRIVATE_INPUT_PATH,
        foreach_private_input_field
    )(&run_info);

    let output = run_info
        .env_or(
            "ZKWASM_OUTPUT",
            "/tmp/output",
        );

    let params = run_info
        .env_or(
            "ZKWASM_PARAMS",
            "/tmp/params",
        );

    match run_info.run_type {
        Execute => benchmarkable! {
            run(zkwasm_command("dry-run")
                .arg("--public").arg(public_input.clone())
                .arg("--private").arg(private_input.clone())
                .arg("--output").arg(output.clone()));
        },
        Prove => benchmarkable! {
            run(zkwasm_command("prove")
                .arg("--public").arg(public_input.clone())
                .arg("--private").arg(private_input.clone())
                .arg("--output").arg(output.clone()));
        },
        Verify => {
            run(zkwasm_command("prove")
                .arg("--public")
                .arg(public_input)
                .arg("--private")
                .arg(private_input)
                .arg("--output")
                .arg(output.clone()));

            benchmarkable! {
                run(Command::new("zkwasm-cli")
                    .arg("--params").arg(params.clone())
                    .arg("prog").arg("verify")
                    .arg("--output").arg(output.clone()));
            }
        }
    }
}
