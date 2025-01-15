use zkvms_host_io::{read_args, RunType::{Execute, Prove, Verify}};
use std::io::{self, Write};
use std::process::{Command, Stdio};

static K: &str = "19";
static SCHEME: &str = "shplonk";

type Input = (Vec<Vec<bool>>, u32, Vec<Vec<u32>>);

fn build_input((graph, colors, coloring): &Input) -> String {
    let mut ret = String::new();
    for vec in graph {
        for b in vec {
            ret.push_str(&(*b as i32).to_string());
            ret.push_str(":i64,");
        }
    }
    ret.push_str(&colors.to_string());
    ret.push_str(":i64,");
    for vec in coloring {
        for c in vec {
            ret.push_str(&c.to_string());
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

    run(zkwasm_command("setup")
        .arg("-k").arg(K)
        .arg("--scheme").arg(SCHEME));

    let input = build_input(&run_info.input);

    match run_info.run_type {
        Execute => {
            run(zkwasm_command("dry-run")
                .arg("--private").arg(input)
                .arg("--output").arg("./output"));
        },
        Prove => {
            run(zkwasm_command("prove")
                .arg("--private").arg(input)
                .arg("--output").arg("./output"));
        },
        Verify => {
            run(zkwasm_command("prove")
                .arg("--private").arg(input)
                .arg("--output").arg("./output"));

            run(Command::new("zkwasm-cli")
                .arg("--params").arg("./params")
                .arg("prog").arg("verify")
                .arg("--output").arg("./output"));
        },
    }
}
