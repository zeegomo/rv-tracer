pub mod air;
pub mod executor;
pub mod prover;
pub mod trace;
use executor::Program;
use rvsim::elf::Elf32;
use std::time::Instant;
use winterfell::ProverError;

pub type Felem = winterfell::math::fields::f64::BaseElement;

use air::{Inputs, Segment, SegmentConfig};
use trace::TraceTable;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f64::BaseElement,
    ProofOptions, Prover, StarkProof, Trace, VerifierError,
};

pub fn prove<H: ElementHasher<BaseField = BaseElement>>(
    trace: TraceTable<BaseElement>,
    options: ProofOptions,
    inputs: Inputs,
) -> Result<StarkProof, ProverError> {
    let prover = prover::RiscvProver::<H>::new(options, inputs);
    let now = Instant::now();
    let proof = prover.prove(trace)?;
    log::debug!("Generated proof in {} ms", now.elapsed().as_millis());
    Ok(proof)
}

pub fn prove_from_elf<H: ElementHasher<BaseField = BaseElement>>(
    elf: Elf32,
    options: ProofOptions,
    segment_config: SegmentConfig,
) -> Result<(Vec<StarkProof>, usize), ProverError> {
    log::debug!(
        "Generating proof for riscv program\n\
        ---------------------"
    );
    // generate execution trace
    let now = Instant::now();
    let program: Program = (&elf).into();
    let traces = executor::exec(&program, segment_config);

    let trace_width = traces[0].get_info().width();
    let trace_length = traces[0].length();
    log::debug!(
        "Generated execution trace of {} registers and 2^{} steps in {} ms",
        trace_width,
        (trace_length * traces.len()).ilog2(),
        now.elapsed().as_millis()
    );
    let n_cycles = traces.iter().map(|t| t.length() - 1).sum::<usize>();
    let proofs = traces
        .into_iter()
        .enumerate()
        .map(|(segment_n, trace)| {
            let inputs = Inputs {
                program: program.clone(),
                segment: Segment {
                    segment_n: segment_n as u32,
                },
                n_cycles,
            };
            prove::<H>(trace, options.clone(), inputs)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((proofs, n_cycles))
}

pub fn verify<H: ElementHasher<BaseField = BaseElement>>(
    proof: StarkProof,
    inputs: Inputs,
) -> Result<(), VerifierError> {
    let now = Instant::now();
    winterfell::verify::<air::RiscvAir, H, DefaultRandomCoin<H>>(proof, inputs)?;
    log::debug!("Verified proof in {} ms", now.elapsed().as_millis());
    Ok(())
}
