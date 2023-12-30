mod common;

use rv_tracer::{prove_from_elf, verify};

use common::{Blake3_192, PROOF_OPTIONS};

const LOOP_ELF: &[u8] = include_bytes!("../../loop/loop.bin");

#[test]
fn prove_and_verify_loop() {
    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();

    let proof = prove_from_elf::<Blake3_192>(elf, PROOF_OPTIONS.clone()).unwrap();
    verify::<Blake3_192>(proof).unwrap();
    panic!();
}
