use criterion::{black_box, criterion_group, criterion_main, Criterion};
use miden_processor::QuadExtension;
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    prove, verify,
};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions, Trace};
// TODO add generate gtrace

const LOOP_ELF: &[u8] = include_bytes!("../../examples/loop/loop.bin");
const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 16;
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

pub fn loop_15(c: &mut Criterion) {
    c.bench_function("cpu loop prove", |b| {
        b.iter(|| {
            let elf = Elf32::parse(LOOP_ELF).unwrap();
            let program = rv_tracer::executor::Program::from(&elf);
            let trace = rv_tracer::executor::exec(&program, SegmentConfig::Single)
                .pop()
                .unwrap();
            let inputs = Inputs {
                program: program.clone(),
                segment: Segment { segment_n: 0 },
                n_cycles: trace.length() - 1,
            };
            prove::<Blake3_192, QuadExtension<BaseElement>>(
                trace,
                black_box(PROOF_OPTIONS.clone()),
                inputs,
            )
        })
    });

    c.bench_function("cpu loop verify", |b| {
        let elf = Elf32::parse(LOOP_ELF).unwrap();
        let program = rv_tracer::executor::Program::from(&elf);
        let trace = rv_tracer::executor::exec(&program, SegmentConfig::Single)
            .pop()
            .unwrap();
        let inputs = Inputs {
            program: program.clone(),
            segment: Segment { segment_n: 0 },
            n_cycles: trace.length() - 1,
        };
        let proof = prove::<Blake3_192, QuadExtension<BaseElement>>(
            trace,
            black_box(PROOF_OPTIONS.clone()),
            inputs.clone(),
        )
        .unwrap();
        b.iter(|| verify::<Blake3_192>(proof.clone(), inputs.clone()))
    });

    c.bench_function("cpu loop trace generation", |b| {
        b.iter(|| {
            rv_tracer::executor::exec(
                &(&Elf32::parse(LOOP_ELF).unwrap()).into(),
                SegmentConfig::Single,
            )
        });
    });
}

pub fn fibonacci_1000(c: &mut Criterion) {
    c.bench_function("cpu fibonacci prove", |b| {
        b.iter(|| {
            let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
            let program = rv_tracer::executor::Program::from(&elf);
            let trace = rv_tracer::executor::exec(&program, SegmentConfig::Single)
                .pop()
                .unwrap();
            let inputs = Inputs {
                program: program.clone(),
                segment: Segment { segment_n: 0 },
                n_cycles: trace.length() - 1,
            };
            prove::<Blake3_192, QuadExtension<BaseElement>>(
                trace,
                black_box(PROOF_OPTIONS.clone()),
                inputs,
            )
        })
    });

    c.bench_function("cpu fibonacci verify", |b| {
        let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
        let program = rv_tracer::executor::Program::from(&elf);
        let trace = rv_tracer::executor::exec(&program, SegmentConfig::Single)
            .pop()
            .unwrap();
        let inputs = Inputs {
            program: program.clone(),
            segment: Segment { segment_n: 0 },
            n_cycles: trace.length() - 1,
        };
        let proof = prove::<Blake3_192, QuadExtension<BaseElement>>(
            trace,
            black_box(PROOF_OPTIONS.clone()),
            inputs.clone(),
        )
        .unwrap();
        b.iter(|| verify::<Blake3_192>(proof.clone(), inputs.clone()))
    });

    c.bench_function("cpu fibonacci trace generation", |b| {
        b.iter(|| {
            rv_tracer::executor::exec(
                &(&Elf32::parse(FIBONACCI_ELF).unwrap()).into(),
                SegmentConfig::Single,
            )
        });
    });
}

criterion_group!(cpu, loop_15, fibonacci_1000);
criterion_main!(cpu);
