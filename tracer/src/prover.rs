use crate::{air::RiscvAir, executor::Program, trace::TraceTable};
use core::marker::PhantomData;
use std::f32::MAX;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f64::BaseElement,
    ProofOptions, Prover, ProverError, StarkProof,
};

const DEFAULT_SEGMENT_SIZE: usize = 1 << 15; // 32K

pub struct RiscvProver<H: ElementHasher, const MAX_SEGMENT_SIZE: usize = DEFAULT_SEGMENT_SIZE> {
    options: ProofOptions,
    program: Program,
    _hasher: PhantomData<H>,
}

impl<const MAX_SEGMENT_SIZE: usize, H> RiscvProver<H, MAX_SEGMENT_SIZE>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    pub fn new(options: ProofOptions, program: Program) -> Self {
        Self {
            options,
            _hasher: PhantomData,
            program,
        }
    }

    pub fn prove_with_split(
        &self,
        trace: <Self as Prover>::Trace,
        segment_size: usize,
    ) -> Result<(Vec<StarkProof>, Vec<StarkProof>), ProverError> {
        assert!(segment_size.is_power_of_two());

        let traces = trace.split(segment_size);
        assert!(!traces.is_empty());
        println!("proving individual segments");
        let proofs = traces
            .clone()
            .into_iter()
            .map(|trace| {
                println!("proving one");
                self.prove(trace)
            })
            .collect::<Result<Vec<StarkProof>, ProverError>>()?;
        println!("proving links");
        let link_proofs = traces
            .windows(2)
            .map(|traces| self.prove_link(traces[0].clone(), traces[1].clone()))
            .collect::<Result<_, _>>()?;

        Ok((proofs, link_proofs))
    }
}

impl<const MAX_SEGMENT_SIZE: usize, H: ElementHasher> Prover for RiscvProver<H, MAX_SEGMENT_SIZE>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    type BaseField = BaseElement;
    type Air = RiscvAir;
    type Trace = TraceTable<BaseElement>;
    type HashFn = H;
    type RandomCoin = DefaultRandomCoin<Self::HashFn>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> <Self::Air as winterfell::Air>::PublicInputs {
        self.program.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
