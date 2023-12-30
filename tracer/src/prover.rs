use crate::{air::RiscvAir, trace::TraceTable};
use core::marker::PhantomData;
use winterfell::{
    crypto::{DefaultRandomCoin, ElementHasher},
    math::{fields::f64::BaseElement, FieldElement},
    // matrix::ColMatrix,
    AuxTraceRandElements,
    ConstraintCompositionCoefficients,
    ProofOptions,
    Prover,
    TraceInfo,
};

pub struct RiscvProver<H: ElementHasher> {
    options: ProofOptions,
    _hasher: PhantomData<H>,
}

impl<H: ElementHasher> RiscvProver<H> {
    pub fn new(options: ProofOptions) -> Self {
        Self {
            options,
            _hasher: PhantomData,
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
    // type TraceLde<E: FieldElement<BaseField = Self::BaseField>> = DefaultTraceLde<E, Self::HashFn>;
    // type ConstraintEvaluator<'a, E: FieldElement<BaseField = Self::BaseField>> =
    //     DefaultConstraintEvaluator<'a, Self::Air, E>;

    fn get_pub_inputs(&self, _trace: &Self::Trace) -> <Self::Air as winterfell::Air>::PublicInputs {
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }

    // fn new_trace_lde<E: FieldElement<BaseField = Self::BaseField>>(
    //     &self,
    //     trace_info: &TraceInfo,
    //     main_trace: &ColMatrix<Self::BaseField>,
    //     domain: &StarkDomain<Self::BaseField>,
    // ) -> (Self::TraceLde<E>, TracePolyTable<E>) {
    //     DefaultTraceLde::new(trace_info, main_trace, domain)
    // }

    // fn new_evaluator<'a, E: FieldElement<BaseField = Self::BaseField>>(
    //     &self,
    //     air: &'a Self::Air,
    //     aux_rand_elements: AuxTraceRandElements<E>,
    //     composition_coefficients: ConstraintCompositionCoefficients<E>,
    // ) -> Self::ConstraintEvaluator<'a, E> {
    //     DefaultConstraintEvaluator::new(air, aux_rand_elements, composition_coefficients)
    // }
}
