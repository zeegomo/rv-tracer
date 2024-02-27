use criterion::{black_box, criterion_group, criterion_main, Criterion};
use miden_processor::QuadExtension;
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    prove, prove_segmented, verify, verify_segmented,
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

    c.bench_function("cpu fibonacci prove 1 segment (32K)", |b| {
        b.iter(|| {
            let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
            let program = rv_tracer::executor::Program::from(&elf);
            let traces = rv_tracer::executor::exec(
                &program,
                SegmentConfig::Split {
                    segment_len: 1 << 15,
                },
            );
            let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
            let inputs = Inputs {
                program: program.clone(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };
            prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
                traces,
                black_box(PROOF_OPTIONS.clone()),
                inputs,
            )
        })
    });

    c.bench_function("cpu fibonacci prove segment 8 segments (4K) ", |b| {
        b.iter(|| {
            let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
            let program = rv_tracer::executor::Program::from(&elf);
            let traces = rv_tracer::executor::exec(
                &program,
                SegmentConfig::Split {
                    segment_len: 1 << 12,
                },
            );
            let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
            let inputs = Inputs {
                program: program.clone(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };
            prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
                traces,
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

    c.bench_function("cpu fibonacci verify 1 segment (32K)", |b| {
        let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
        let program = rv_tracer::executor::Program::from(&elf);
        let traces = rv_tracer::executor::exec(
            &program,
            SegmentConfig::Split {
                segment_len: 1 << 15,
            },
        );
        let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
        let inputs = Inputs {
            program: program.clone(),
            segment: Segment { segment_n: 0 },
            n_cycles,
        };
        let (proofs, link_proofs) = prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
            traces,
            black_box(PROOF_OPTIONS.clone()),
            inputs.clone(),
        )
        .unwrap();
        b.iter(|| {
            verify_segmented::<Blake3_192>(proofs.clone(), link_proofs.clone(), inputs.clone())
        })
    });

    c.bench_function("cpu fibonacci verify 8 segments (4K)", |b| {
        let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
        let program = rv_tracer::executor::Program::from(&elf);
        let traces = rv_tracer::executor::exec(
            &program,
            SegmentConfig::Split {
                segment_len: 1 << 11,
            },
        );
        let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
        let inputs = Inputs {
            program: program.clone(),
            segment: Segment { segment_n: 0 },
            n_cycles,
        };
        let (proofs, link_proofs) = prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
            traces,
            black_box(PROOF_OPTIONS.clone()),
            inputs.clone(),
        )
        .unwrap();
        b.iter(|| {
            verify_segmented::<Blake3_192>(proofs.clone(), link_proofs.clone(), inputs.clone())
        })
    });

    c.bench_function("cpu fibonacci trace generation", |b| {
        b.iter(|| {
            rv_tracer::executor::exec(
                &(&Elf32::parse(FIBONACCI_ELF).unwrap()).into(),
                SegmentConfig::Single,
            )
        });
    });

    c.bench_function("cpu fibonacci trace generation 1 segment", |b| {
        b.iter(|| {
            rv_tracer::executor::exec(
                &(&Elf32::parse(FIBONACCI_ELF).unwrap()).into(),
                SegmentConfig::Split {
                    segment_len: 1 << 15,
                },
            )
        });
    });

    c.bench_function("cpu fibonacci trace generation 8 segments (4K)", |b| {
        b.iter(|| {
            rv_tracer::executor::exec(
                &(&Elf32::parse(FIBONACCI_ELF).unwrap()).into(),
                SegmentConfig::Split {
                    segment_len: 1 << 11,
                },
            )
        });
    });
}

criterion_group!(cpu_short, loop_15);
criterion_group!(
    name = cpu_long;
    config = Criterion::default().sample_size(10);
    targets = fibonacci_1000
);
criterion_main!(cpu_short, cpu_long);
