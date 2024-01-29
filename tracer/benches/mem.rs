use clap::Parser;
use once_cell::sync::Lazy;
use rv_tracer::{executor::Program, prove, verify};
use rvsim::elf::Elf32;
use serde::{Deserialize, Serialize};
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

fn format(name: &str, val: usize) -> String {
    let mut val = val as f64;
    let mut unit = "B";
    if val > 1024.0 {
        val = val / 1024.0;
        unit = "kB";
    }

    // if val > 1024.0 {
    //     val = val / 1024.0;
    //     unit = "MB";
    // }
    format!("{name}\t\t peak mem usage:   {val} {unit}")
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

    (prove, verify, exec)
}

fn save_baseline(baseline: &str, group_id: &str, function_id: &str, result: usize) {
    let cbenchmark = CBenchmark {
        group_id: group_id.to_string(),
        function_id: Some(function_id.into()),
        value_str: None,
        throughput: None,
        full_id: group_id.to_string(),
        directory_name: group_id.to_string(),
    };
    let confidence_interval = CONFIDENCE;
    let point_estimate = result as f64;
    let standard_error = 0.0;
    let cestimates = CEstimates {
        mean: CStats {
            confidence_interval,
            point_estimate,
            standard_error,
        },
        median: CStats {
            confidence_interval,
            point_estimate,
            standard_error,
        },
        median_abs_dev: CStats {
            confidence_interval,
            point_estimate: 0.0,
            standard_error: 0.0,
        },
        slope: None,
        std_dev: CStats {
            confidence_interval,
            point_estimate: 0.0,
            standard_error: 0.0,
        },
    };

    std::fs::create_dir_all(format!("criterion/{group_id}/{function_id}/{baseline}")).unwrap();
    std::fs::write(
        format!(
            "criterion/{group_id}/{function_id}/{baseline}/estimates.json
        "
        ),
        serde_json::to_string(&cestimates).unwrap(),
    )
    .unwrap();
    std::fs::write(
        format!(
            "criterion/{group_id}/{function_id}/{baseline}/benchmark.json
    "
        ),
        serde_json::to_string(&cbenchmark).unwrap(),
    )
    .unwrap();
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Save results as a baseline for future comparisons
    #[arg(long = "save-baseline")]
    save_baseline: Option<String>,
    // ignore this, it's just to make it compatible with rust benchmark cli
    #[arg(long)]
    bench: bool,
}

fn main() {
    let args = Args::parse();
    let (loop_prove, loop_verify, loop_exec) = loop_15();
    let (fibonacci_prove, fibonacci_verify, fibonacci_exec) = fibonacci_1000();
    if let Some(baseline) = &args.save_baseline {
        save_baseline(baseline, "loop".into(), "prove", loop_prove);
        save_baseline(baseline, "loop".into(), "verify", loop_verify);
        save_baseline(baseline, "loop".into(), "exec", loop_exec);
        save_baseline(baseline, "fibonacci".into(), "prove", fibonacci_prove);
        save_baseline(baseline, "fibonacci".into(), "verify", fibonacci_verify);
        save_baseline(baseline, "fibonacci".into(), "exec", fibonacci_exec);
        return;
    } else {
        println!("loop");
        println!("{}", format("prove", loop_prove));
        println!("{}", format("verify", loop_verify));
        println!("{}", format("exec", loop_exec));
        println!("\nfibonacci");
        println!("{}", format("prove", fibonacci_prove));
        println!("{}", format("verify", fibonacci_verify));
        println!("{}", format("exec", fibonacci_exec));
    }
}

// Taken from https://github.com/BurntSushi/critcmp/blob/master/src/data.rs

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CBenchmark {
    pub group_id: String,
    pub function_id: Option<String>,
    pub value_str: Option<String>,
    pub throughput: Option<CThroughput>,
    pub full_id: String,
    pub directory_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct CThroughput {
    pub bytes: Option<u64>,
    pub elements: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CEstimates {
    pub mean: CStats,
    pub median: CStats,
    pub median_abs_dev: CStats,
    pub slope: Option<CStats>,
    pub std_dev: CStats,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CStats {
    pub confidence_interval: CConfidenceInterval,
    pub point_estimate: f64,
    pub standard_error: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize, Copy)]
pub struct CConfidenceInterval {
    pub confidence_level: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
}

const CONFIDENCE: CConfidenceInterval = CConfidenceInterval {
    confidence_level: 0.95,
    lower_bound: 0.0,
    upper_bound: 0.0,
};
