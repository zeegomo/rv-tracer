use crate::{air::RiscvAir, executor::Program, trace::TraceTable};
use core::marker::PhantomData;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f64::BaseElement,
    ProofOptions, Prover,
};

pub struct RiscvProver<H: ElementHasher> {
    options: ProofOptions,
    program: Program,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher> RiscvProver<H> {
    pub fn new(options: ProofOptions, program: Program) -> Self {
        Self {
            options,
            _hasher: PhantomData,
            program,
        }
    }
}

impl<H: ElementHasher> Prover for RiscvProver<H>
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
