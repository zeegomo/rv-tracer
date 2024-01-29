use once_cell::sync::Lazy;
use rv_tracer::{executor::Program, prove, verify};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions};
// TODO add generate gtrace

const LOOP_ELF: &[u8] = include_bytes!("../../examples/loop/loop.bin");
const FIBONACCI_ELF: &[u8] = include_bytes!("../../examples/fibonacci/fibonacci.bin");

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

use peak_alloc::PeakAlloc;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

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

fn format(name: &str, val: usize, baseline: usize) -> String {
    let change = (val as f64 / baseline as f64 - 1.0) * 100.0;
    format!(
        "
        {}          time:   [0 kB {} kB 0kB]
                    change: [+0% {}% +0%] (p = 0.00 < 0.05)
                    No change in performance detected.",
        name, val, change
    )
}

fn loop_15() -> (usize, usize, usize) {
    let elf = Elf32::parse(LOOP_ELF).unwrap();
    let program = rv_tracer::executor::Program::from(&elf);

    bench(program)
}

fn fibonacci_1000() -> (usize, usize, usize) {
    let elf = Elf32::parse(FIBONACCI_ELF).unwrap();
    let program = rv_tracer::executor::Program::from(&elf);

    bench(program)
}

fn bench(program: Program) -> (usize, usize, usize) {
    PEAK_ALLOC.reset_peak_usage();
    let trace = rv_tracer::executor::exec(&program);
    let exec = PEAK_ALLOC.peak_usage();

    PEAK_ALLOC.reset_peak_usage();
    let proof = prove::<Blake3_192>(trace, PROOF_OPTIONS.clone(), program.clone()).unwrap();
    let prove = PEAK_ALLOC.peak_usage();

    PEAK_ALLOC.reset_peak_usage();
    verify::<Blake3_192>(proof, program).unwrap();
    let verify = PEAK_ALLOC.peak_usage();

    (prove / 1024, verify / 1024, exec / 1024)
}

fn load_baseline(name: &str) -> Option<(usize, usize, usize)> {
    let baseline = std::fs::read_to_string(format!("{}.baseline", name)).ok()?;
    let (prove, verify, exec): (usize, usize, usize) = serde_json::from_str(&baseline).ok()?;
    Some((prove, verify, exec))
}

fn main() {
    let (prove, verify, exec) = loop_15();
    let (baseline_prove, baseline_verify, baseline_exec) =
        load_baseline("loop").unwrap_or_default();
    println!("{}", format(&format!("loop-prove"), prove, baseline_prove));
    println!(
        "{}",
        format(&format!("loop-verify"), verify, baseline_verify)
    );
    println!("{}", format(&format!("loop-exec"), exec, baseline_exec));
    std::fs::write(
        "loop.baseline",
        serde_json::to_string(&(prove, verify, exec)).unwrap(),
    )
    .unwrap();

    let (prove, verify, exec) = fibonacci_1000();
    let (baseline_prove, baseline_verify, baseline_exec) = load_baseline("fib").unwrap_or_default();
    println!("{}", format(&format!("fib-prove"), prove, baseline_prove));
    println!(
        "{}",
        format(&format!("fib-verify"), verify, baseline_verify)
    );
    println!("{}", format(&format!("fib-exec"), exec, baseline_exec));
    std::fs::write(
        "fib.baseline",
        serde_json::to_string(&(prove, verify, exec)).unwrap(),
    )
    .unwrap();
}
