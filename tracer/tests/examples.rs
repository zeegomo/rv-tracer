mod common;

use miden_processor::QuadExtension;
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::Program,
    prove_from_elf, verify, verify_segmented,
};
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions};

use common::{Blake3_192, PROOF_OPTIONS};

const LOOP_ELF: &[u8] = include_bytes!("../../examples/loop/loop.bin");
const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

const SEGMENT_LEN: u32 = 1 << 12;
const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 16;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub static QUAD_PROOF_OPTIONS: Lazy<ProofOptions> = Lazy::new(|| {
    ProofOptions::new(
        NUM_QUERIES,
        BLOWUP_FACTOR,
        GRINDING_FACTOR,
        FieldExtension::Quadratic,
        FRI_FOLDING_FACTOR,
        FRI_REMAINDER_MAX_DEGREE,
    )
});

#[test]
fn prove_and_verify_loop() {
    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();
    let program = Program::from(&elf);
    let (proof, _link_proofs, n_cycles) = prove_from_elf::<Blake3_192, BaseElement>(
        elf,
        PROOF_OPTIONS.clone(),
        SegmentConfig::Single,
    )
    .unwrap();
    verify::<Blake3_192>(
        proof,
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
    let (proof, _link_proofs, n_cycles) = prove_from_elf::<Blake3_192, BaseElement>(
        elf,
        PROOF_OPTIONS.clone(),
        SegmentConfig::Single,
    )
    .unwrap();
    verify::<Blake3_192>(
        proof,
        Inputs {
            program,
            segment: Segment { segment_n: 0 },
            n_cycles,
        },
    )
    .unwrap();
}

#[test]
fn prove_and_verify_loop_split() {
    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();
    let program = Program::from(&elf);
    let (proofs, link_proofs, n_cycles) = prove_from_elf::<Blake3_192, QuadExtension<BaseElement>>(
        elf,
        QUAD_PROOF_OPTIONS.clone(),
        SegmentConfig::Split {
            segment_len: SEGMENT_LEN,
        },
    )
    .unwrap();
    verify_segmented::<Blake3_192>(
        proofs,
        link_proofs,
        Inputs {
            program,
            segment: Segment { segment_n: 0 },
            n_cycles,
        },
    )
    .unwrap();
}

#[test]
fn prove_and_verify_fibonacci_split() {
    let elf = rvsim::elf::Elf32::parse(FIBONACCI_ELF).unwrap();
    let program = Program::from(&elf);
    let (proofs, link_proofs, n_cycles) = prove_from_elf::<Blake3_192, QuadExtension<BaseElement>>(
        elf,
        QUAD_PROOF_OPTIONS.clone(),
        SegmentConfig::Split {
            segment_len: SEGMENT_LEN,
        },
    )
    .unwrap();
    verify_segmented::<Blake3_192>(
        proofs,
        link_proofs,
        Inputs {
            program,
            segment: Segment { segment_n: 0 },
            n_cycles,
        },
    )
    .unwrap();
}
