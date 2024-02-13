use dhat::HeapStats;
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::{exec, Program},
    prove, verify,
};
use rvsim::elf::Elf32;
use winterfell::{math::fields::f64::BaseElement, FieldExtension, ProofOptions, StarkProof, Trace};

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
    let file_path = "fibonacci_1000_proof";

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).unwrap();
            let mut proof = std::fs::read(file_path).unwrap();
            let n_cycles =
                usize::from_le_bytes(proof.split_off(proof.len() - 8).try_into().unwrap());
            let proof = StarkProof::from_bytes(&proof).unwrap();
            let inputs = Inputs {
                program: fibonacci_1000(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };
            verify::<Blake3_192>(proof, inputs).unwrap();
            let HeapStats {
                total_blocks,
                total_bytes,
                max_blocks,
                max_bytes,
                ..
            } = dhat::HeapStats::get();
            println!("out=[{total_blocks},{total_bytes},{max_blocks},{max_bytes}]");
        }
        Ok(ForkResult::Child) => {
            // we need to prove the program in a different process so that it does not interfere
            // with verify profiling
            let trace = exec(&fibonacci_1000(), SegmentConfig::Single)
                .pop()
                .unwrap();
            let n_cycles = trace.length() - 1;
            let inputs = Inputs {
                program: fibonacci_1000(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };
            let proof = prove::<Blake3_192>(trace, PROOF_OPTIONS.clone(), inputs).unwrap();
            std::fs::write(
                file_path,
                proof
                    .to_bytes()
                    .into_iter()
                    .chain(n_cycles.to_le_bytes())
                    .collect::<Vec<_>>(),
            )
            .unwrap();
            // exit the child process so that we don't save profilings here
            unsafe { libc::_exit(0) };
        }
        Err(_) => panic!("Fork failed"),
    }
}
