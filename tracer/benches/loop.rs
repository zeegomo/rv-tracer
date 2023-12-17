use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rv_tracer::{prove, verify};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f128::BaseElement, FieldExtension, ProofOptions};
// TODO add generate gtrace

const LOOP_ELF: &[u8] = include_bytes!("../../loop/loop.bin");

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

pub const PROOF_OPTIONS: ProofOptions = ProofOptions::new(
    NUM_QUERIES,
    BLOWUP_FACTOR,
    GRINDING_FACTOR,
    FieldExtension::None,
    FRI_FOLDING_FACTOR,
    FRI_REMAINDER_MAX_DEGREE,
);

pub fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("loop prove", |b| {
        let trace = rv_tracer::sim::sim(Elf32::parse(LOOP_ELF).unwrap());
        b.iter(|| prove::<Blake3_192>(trace.clone(), black_box(PROOF_OPTIONS)))
    });

    c.bench_function("loop verify", |b| {
        let trace = rv_tracer::sim::sim(Elf32::parse(LOOP_ELF).unwrap());
        let proof = prove::<Blake3_192>(trace.clone(), black_box(PROOF_OPTIONS)).unwrap();
        b.iter(|| verify::<Blake3_192>(proof.clone()))
    });

    c.bench_function("loop trace generation", |b| {
        b.iter(|| rv_tracer::sim::sim::<BaseElement>(Elf32::parse(black_box(LOOP_ELF)).unwrap()));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
