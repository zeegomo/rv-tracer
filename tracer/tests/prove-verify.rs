use rv_tracer::{prove, verify};
use winterfell::{math::fields::f128::BaseElement, FieldExtension, ProofOptions};
type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

const LOOP_ELF: &[u8] = include_bytes!("../../loop/loop.bin");

#[test]
fn prove_and_verify_loop() {
    const NUM_QUERIES: usize = 10;
    const BLOWUP_FACTOR: usize = 32;
    const GRINDING_FACTOR: u32 = 5;
    const FRI_FOLDING_FACTOR: usize = 4;
    const FRI_REMAINDER_MAX_DEGREE: usize = 255;

    let elf = rvsim::elf::Elf32::parse(LOOP_ELF).unwrap();

    let proof = prove::<Blake3_192>(
        elf,
        ProofOptions::new(
            NUM_QUERIES,
            BLOWUP_FACTOR,
            GRINDING_FACTOR,
            FieldExtension::None,
            FRI_FOLDING_FACTOR,
            FRI_REMAINDER_MAX_DEGREE,
        ),
    );
    verify::<Blake3_192>(proof).unwrap();
}
