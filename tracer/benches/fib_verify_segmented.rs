use dhat::HeapStats;
use miden_processor::QuadExtension;
use nix::{
    sys::wait::waitpid,
    unistd::{fork, ForkResult},
};
use once_cell::sync::Lazy;
use rv_tracer::{
    air::{Inputs, Segment, SegmentConfig},
    executor::{exec, Program},
    prove_segmented, verify_segmented,
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

const SEGMENT_LEN: usize = 1 << 12;

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
    let proofs_prefix = "proof_";
    let link_proofs_prefix = "link_proof_";
    let info_path = "info";

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).unwrap();
            let mut info = std::fs::read(info_path).unwrap();
            let n_cycles = usize::from_le_bytes(info.split_off(8).try_into().unwrap());
            let n_proofs = usize::from_le_bytes(info.try_into().unwrap());

            let mut proofs = Vec::with_capacity(n_proofs);
            for i in 0..n_proofs {
                proofs.push(
                    serde_json::from_slice(
                        &std::fs::read(format!("{}{}", proofs_prefix, i)).unwrap(),
                    )
                    .unwrap(),
                );
            }

            let mut link_proofs = Vec::with_capacity(n_proofs - 1);
            for i in 0..n_proofs - 1 {
                link_proofs.push(
                    serde_json::from_slice(
                        &std::fs::read(format!("{}{}", link_proofs_prefix, i)).unwrap(),
                    )
                    .unwrap(),
                );
            }

            let inputs = Inputs {
                program: fibonacci_1000(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };

            verify_segmented::<Blake3_192>(proofs, link_proofs, inputs).unwrap();
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
            let traces = exec(
                &fibonacci_1000(),
                SegmentConfig::Split {
                    segment_len: SEGMENT_LEN as u32,
                },
            );
            let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
            let inputs = Inputs {
                program: fibonacci_1000(),
                segment: Segment { segment_n: 0 },
                n_cycles,
            };
            let (proofs, link_proofs) = prove_segmented::<Blake3_192, QuadExtension<BaseElement>>(
                traces,
                PROOF_OPTIONS.clone(),
                inputs.clone(),
            )
            .unwrap();

            for (i, proof) in proofs.iter().enumerate() {
                std::fs::write(
                    format!("{}{}", proofs_prefix, i),
                    serde_json::to_string(proof).unwrap(),
                )
                .unwrap();
            }
            for (i, link_proof) in link_proofs.iter().enumerate() {
                std::fs::write(
                    format!("{}{}", link_proofs_prefix, i),
                    serde_json::to_string(link_proof).unwrap(),
                )
                .unwrap();
            }

            verify_segmented::<Blake3_192>(proofs.clone(), link_proofs.clone(), inputs.clone())
                .unwrap();

            std::fs::write(
                info_path,
                proofs
                    .len()
                    .to_le_bytes()
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
