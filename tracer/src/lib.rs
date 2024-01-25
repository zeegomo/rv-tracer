pub mod air;
pub mod executor;
pub mod prover;
pub mod trace;
use executor::Program;
use rvsim::elf::Elf32;
use std::time::Instant;
use winterfell::ProverError;

pub type Felem = winterfell::math::fields::f64::BaseElement;

use trace::TraceTable;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f64::BaseElement,
    ProofOptions, Prover, StarkProof, Trace, VerifierError,
};

pub fn prove<H: ElementHasher<BaseField = BaseElement>>(
    trace: TraceTable<BaseElement>,
    options: ProofOptions,
    program: Program,
) -> Result<StarkProof, ProverError> {
    // generate the proof
    let prover = prover::RiscvProver::<H>::new(options, program);
    let now = Instant::now();
    let proof = prover.prove(trace)?;
    log::debug!("Generated proof in {} ms", now.elapsed().as_millis());
    Ok(proof)
}

pub fn prove_from_elf<H: ElementHasher<BaseField = BaseElement>>(
    elf: Elf32,
    options: ProofOptions,
) -> Result<StarkProof, ProverError> {
    log::debug!(
        "Generating proof for riscv program\n\
        ---------------------"
    );
    // generate execution trace
    let now = Instant::now();
    let program: Program = (&elf).into();
    let trace = executor::exec(&program);

    let trace_width = trace.get_info().width();
    let trace_length = trace.length();
    log::debug!(
        "Generated execution trace of {} registers and 2^{} steps in {} ms",
        trace_width,
        trace_length.ilog2(),
        now.elapsed().as_millis()
    );

    prove::<H>(trace, options, program)
}

pub fn verify<H: ElementHasher<BaseField = BaseElement>>(
    proof: StarkProof,
    program: Program,
) -> Result<(), VerifierError> {
    let now = Instant::now();
    winterfell::verify::<air::RiscvAir, H, DefaultRandomCoin<H>>(proof, program)?;
    log::debug!("Verified proof in {} ms", now.elapsed().as_millis());
    Ok(())
}
