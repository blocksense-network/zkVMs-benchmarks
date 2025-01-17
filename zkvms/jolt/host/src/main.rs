use zkvms_host_io::{read_args, RunType::{ Execute, Prove, Verify }};

type Input = (Vec<Vec<bool>>, u32, Vec<Vec<u32>>);

pub fn main() {
    let run_info = read_args();

    let elf_path = std::env::var("ELF_PATH").expect("ELF PATH is missing");
    let (prove_guest, verify_guest) = guest::guest_closures(elf_path);

    match run_info.run_type {
        Execute => unreachable!(),
        Prove => {
            let (output, _) = prove_guest(run_info.input);
            println!("Prove output: {}", output);
        },
        Verify => {
            let (_, proof) = prove_guest(run_info.input);
            let is_valid = verify_guest(proof);
            println!("Verify is valid: {}", is_valid);
        },
    }
}
