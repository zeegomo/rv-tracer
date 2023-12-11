pub mod air;
pub mod sim;
// pub mod prove;
// pub mod rs;
pub mod prover;
use rvsim::elf::Elf32;
use std::time::Instant;

use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f128::BaseElement,
    ProofOptions, Prover, StarkProof, Trace, VerifierError,
};

pub fn prove<H: ElementHasher<BaseField = BaseElement>>(
    elf: Elf32,
    options: ProofOptions,
) -> StarkProof {
    log::debug!(
        "Generating proof for riscv program\n\
        ---------------------"
    );

    let prover = prover::RiscvProver::<H>::new(options);
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

    // generate the proof
    let now = Instant::now();
    let proof = prover.prove(trace).unwrap();
    log::debug!("Generated proof in {} ms", now.elapsed().as_millis());
    proof
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
