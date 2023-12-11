mod ops;
mod utils;

use winterfell::{
    math::{fields::f128::BaseElement, FieldElement},
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};

pub type BaseField = winterfell::math::fields::f128::BaseElement;

pub struct RiscvAir {
    context: AirContext<BaseElement>,
}

use trace_defs::TRACE_WIDTH;

impl Air for RiscvAir {
    type BaseField = BaseElement;
    type PublicInputs = ();

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: (), options: ProofOptions) -> Self {
        let mut degrees = Vec::new();
        degrees.extend(ops::lui::constraint_degrees());
        // degrees.extend(ops::auipc::constraint_degrees());
        assert_eq!(TRACE_WIDTH, trace_info.width());
        // We also need to specify the exact number of assertions we will place against the
        // execution trace. This number must be the same as the number of items in a vector
        // returned from the get_assertions() method below.
        let num_assertions = 1;
        Self {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let mut index = 0;
        index += ops::lui::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        // ops::auipc::evaluate_transitions(frame, periodic_values, &mut result[index..]);
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // all registers should be 0 at the start of the computation
        // let last_step = self.trace_length() - 1;
        vec![Assertion::single(0, 0, Self::BaseField::ZERO)]
    }
}
