use clap::Parser;
use dhat::HeapStats;
use miden_processor::QuadExtension;
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::{exec, Program},
    prove_segmented,
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

const DEFAULT_SEG_LEN: u32 = 1 << 12;

fn fibonacci_1000() -> Program {
    let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
    Program::from(&elf)
}

fn run_bench(segment_len: u32) -> HeapStats {
    let _profiler = dhat::Profiler::new_heap();
    let traces = exec(&fibonacci_1000(), SegmentConfig::Split { segment_len });
    let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
    let inputs = Inputs {
        program: fibonacci_1000(),
        segment: Segment { segment_n: 0 },
        n_cycles,
    };
    prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
        traces,
        PROOF_OPTIONS.clone(),
        inputs,
    )
    .unwrap();

    dhat::HeapStats::get()
}

#[derive(Parser, Debug)]
struct Args {
    #[clap(short, long, default_value_t = DEFAULT_SEG_LEN)]
    segment_len: u32,

    #[clap(short, long, default_value = "false")]
    bench: bool,
}

fn main() {
    let args = Args::parse();
    println!(
        "prove-peak-{}={}",
        args.segment_len,
        run_bench(args.segment_len).max_bytes
    );
}
