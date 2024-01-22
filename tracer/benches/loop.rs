use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::Lazy;
use rv_tracer::{prove, verify};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions};
// TODO add generate gtrace

const LOOP_ELF: &[u8] = include_bytes!("../../loop/loop.bin");

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

pub static PROOF_OPTIONS: Lazy<ProofOptions> = Lazy::new(|| {
    ProofOptions::new(
        NUM_QUERIES,
        BLOWUP_FACTOR,
        GRINDING_FACTOR,
        FieldExtension::Quadratic,
        FRI_FOLDING_FACTOR,
        FRI_REMAINDER_MAX_DEGREE,
    )
});
pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("loop prove", |b| {
        b.iter(|| {
            let elf = Elf32::parse(LOOP_ELF).unwrap();
            let program = rv_tracer::executor::Program::from(&elf);
            let trace = rv_tracer::executor::exec(&program);
            prove::<Blake3_192>(trace, black_box(PROOF_OPTIONS.clone()), program)
        })
    });

    c.bench_function("loop verify", |b| {
        let elf = Elf32::parse(LOOP_ELF).unwrap();
        let program = rv_tracer::executor::Program::from(&elf);
        let trace = rv_tracer::executor::exec(&program);
        let proof =
            prove::<Blake3_192>(trace, black_box(PROOF_OPTIONS.clone()), program.clone()).unwrap();
        b.iter(|| verify::<Blake3_192>(proof.clone(), program.clone()))
    });

    c.bench_function("loop trace generation", |b| {
        b.iter(|| rv_tracer::executor::exec(&(&Elf32::parse(LOOP_ELF).unwrap()).into()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
