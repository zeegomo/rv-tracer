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
    ProofOptions, StarkProof, Trace, VerifierError,
};

pub fn prove<H: ElementHasher<BaseField = BaseElement>>(
    trace: TraceTable<BaseElement>,
    options: ProofOptions,
    program: Program,
) -> Result<StarkProof, ProverError> {
    let trace_length = trace.length();
    let mut proof = prove_with_split::<H>(trace, options, program, trace_length)?;
    Ok(proof.0.pop().unwrap())
}

pub fn prove_with_split<H: ElementHasher<BaseField = BaseElement>>(
    trace: TraceTable<BaseElement>,
    options: ProofOptions,
    program: Program,
    segment_size: usize,
) -> Result<(Vec<StarkProof>, Vec<StarkProof>), ProverError> {
    // generate the proof
    let prover = prover::RiscvProver::<H>::new(options, program);
    let now = Instant::now();
    let proof = prover.prove_with_split(trace, segment_size)?;
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
    verify_with_split::<H>(vec![proof], Vec::new(), program)
}

pub fn verify_with_split<H: ElementHasher<BaseField = BaseElement>>(
    proofs: Vec<StarkProof>,
    link_proofs: Vec<StarkProof>,
    program: Program,
) -> Result<(), VerifierError> {
    let now = Instant::now();
    for proof in proofs.clone() {
        winterfell::verify::<air::RiscvAir, H, DefaultRandomCoin<H>>(proof, program.clone())?;
    }
    for (proofs, link_proof) in proofs.windows(2).zip(link_proofs) {
        let proof_1 = proofs[0].clone();
        let proof_2 = proofs[1].clone();
        winterfell::verify_split::<air::RiscvAir, H, DefaultRandomCoin<H>>(
            proof_1,
            proof_2,
            link_proof,
            program.clone(),
        )?;
    }
    log::debug!("Verified proof in {} ms", now.elapsed().as_millis());
    Ok(())
}
