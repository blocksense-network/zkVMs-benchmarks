use risc0_binfmt::ProgramBinary;
use risc0_zkos_v1compat::V1COMPAT_ELF;

static ELF: &[u8] = include_bytes!("../../host/src/guest");

// https://github.com/risc0/risc0/blob/fee2e19a3c49b3da492403de5e2d011c890e52de/risc0/build/src/lib.rs#L179-L187
fn main() {
    let binary = ProgramBinary::new(&ELF, V1COMPAT_ELF);
    let elf = binary.encode();
    std::fs::write("../host/src/guest", &elf).expect("couldn't add magic bytes to elf!");
}
