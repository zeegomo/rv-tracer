use dhat::HeapStats;
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::{exec, Program},
    prove,
};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions, Trace};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

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

fn fibonacci_1000() -> Program {
    let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
    Program::from(&elf)
}

fn main() {
    let _profiler = dhat::Profiler::new_heap();
    let trace = exec(&fibonacci_1000(), SegmentConfig::Single)
        .pop()
        .unwrap();
    let inputs = Inputs {
        program: fibonacci_1000(),
        segment: Segment { segment_n: 0 },
        n_cycles: trace.length() - 1,
    };
    prove::<Blake3_192>(trace, PROOF_OPTIONS.clone(), inputs).unwrap();
    let HeapStats {
        total_blocks,
        total_bytes,
        max_blocks,
        max_bytes,
        ..
    } = dhat::HeapStats::get();
    println!("out=[{total_blocks},{total_bytes},{max_blocks},{max_bytes}]");
}
