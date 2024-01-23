mod common;

use rv_tracer::{executor::Program, prove_from_elf, verify};

use common::{Blake3_192, PROOF_OPTIONS};

const LOOP_ELF: &[u8] = include_bytes!("../../examples/loop/loop.bin");
const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

#[test]
fn prove_and_verify_loop() {
    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();
    let program = Program::from(&elf);
    let proof = prove_from_elf::<Blake3_192>(elf, PROOF_OPTIONS.clone()).unwrap();
    verify::<Blake3_192>(proof, program).unwrap();
}

#[test]
fn prove_and_verify_fibonacci() {
    let elf = rvsim::elf::Elf32::parse(FIBONACCI_ELF).unwrap();
    let program = Program::from(&elf);
    let proof = prove_from_elf::<Blake3_192>(elf, PROOF_OPTIONS.clone()).unwrap();
    verify::<Blake3_192>(proof, program).unwrap();
}
