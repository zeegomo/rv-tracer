mod common;

use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::Program,
    prove_from_elf, verify,
};
use winterfell::math::fields::f64::BaseElement;

use common::{Blake3_192, PROOF_OPTIONS};

const LOOP_ELF: &[u8] = include_bytes!("../../examples/loop/loop.bin");
const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

#[test]
fn prove_and_verify_loop() {
    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();
    let program = Program::from(&elf);
    let (mut proof, n_cycles) = prove_from_elf::<Blake3_192, BaseElement>(
        elf,
        PROOF_OPTIONS.clone(),
        SegmentConfig::Single,
    )
    .unwrap();
    verify::<Blake3_192>(
        proof.pop().unwrap(),
        Inputs {
            program,
            segment: Segment { segment_n: 0 },
            n_cycles,
        },
    )
    .unwrap();
}

#[test]
fn prove_and_verify_fibonacci() {
    let elf = rvsim::elf::Elf32::parse(FIBONACCI_ELF).unwrap();
    let program = Program::from(&elf);
    let (mut proof, n_cycles) = prove_from_elf::<Blake3_192, BaseElement>(
        elf,
        PROOF_OPTIONS.clone(),
        SegmentConfig::Single,
    )
    .unwrap();
    verify::<Blake3_192>(
        proof.pop().unwrap(),
        Inputs {
            program,
            segment: Segment { segment_n: 0 },
            n_cycles,
        },
    )
    .unwrap();
}
