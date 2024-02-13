use crate::{
    air::{Inputs, RiscvAir},
    trace::TraceTable,
};
use core::marker::PhantomData;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::fields::f64::BaseElement,
    ProofOptions, Prover,
};

pub struct RiscvProver<H: ElementHasher> {
    options: ProofOptions,
    inputs: Inputs,
    _hasher: PhantomData<H>,
}

impl<H> RiscvProver<H>
where
    H: ElementHasher<BaseField = BaseElement>,
{
    pub fn new(options: ProofOptions, inputs: Inputs) -> Self {
        Self {
            options,
            _hasher: PhantomData,
            inputs,
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
        self.inputs.clone()
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}
