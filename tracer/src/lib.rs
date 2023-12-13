pub mod air;
pub mod sim;
// pub mod prove;
// pub mod rs;
pub mod prover;
use rvsim::elf::Elf32;
use std::time::Instant;
use winterfell::ProverError;

use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f128::BaseElement,
    ProofOptions, Prover, StarkProof, Trace, TraceTable, VerifierError,
};

pub fn prove<H: ElementHasher<BaseField = BaseElement>>(
    trace: TraceTable<BaseElement>,
    options: ProofOptions,
) -> Result<StarkProof, ProverError> {
    // generate the proof
    let prover = prover::RiscvProver::<H>::new(options);
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
    let trace = sim::sim(elf);

    let trace_width = trace.width();
    let trace_length = trace.length();
    log::debug!(
        "Generated execution trace of {} registers and 2^{} steps in {} ms",
        trace_width,
        trace_length.ilog2(),
        now.elapsed().as_millis()
    );

    prove::<H>(trace, options)
}

pub fn verify<H: ElementHasher<BaseField = BaseElement>>(
    proof: StarkProof,
) -> Result<(), VerifierError> {
    let now = Instant::now();
    let acceptable_options =
        winterfell::AcceptableOptions::OptionSet(vec![proof.options().clone()]);

    winterfell::verify::<air::RiscvAir, H, DefaultRandomCoin<H>>(proof, (), &acceptable_options)?;
    log::debug!("Verified proof in {} ms", now.elapsed().as_millis());
    Ok(())
}
