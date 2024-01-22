mod cpu;
mod memory;

use winterfell::{
    math::{fields::f64::BaseElement, ExtensionOf, FieldElement},
    Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions, TraceInfo,
};
pub type BaseField = winterfell::math::fields::f64::BaseElement;

pub struct RiscvAir {
    context: AirContext<BaseElement>,
}

use trace_defs::{AUX_TRACE_WIDTH, LOADING, MAIN_TRACE_WIDTH};

use crate::executor::Program;

impl RiscvAir {
    /// Returns last step of the execution trace.
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

const NUM_ASSERTIONS: usize = 1;

impl Air for RiscvAir {
    type BaseField = BaseElement;
    type PublicInputs = Program;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _program: Program, options: ProofOptions) -> Self {
        assert_eq!(MAIN_TRACE_WIDTH + AUX_TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        degrees.extend(cpu::lui::constraint_degrees());
        degrees.extend(cpu::auipc::constraint_degrees());
        degrees.extend(cpu::addi::constraint_degrees());
        degrees.extend(cpu::jal::constraint_degrees());
        degrees.extend(cpu::jalr::constraint_degrees());
        degrees.extend(cpu::slti::constraint_degrees());

        degrees.extend(memory::get_transition_constraint_degrees());
        // We also need to specify the exact number of assertions we will place against the
        // execution trace. This number must be the same as the number of items in a vector
        // returned from the get_assertions() method below.
        // let num_assertions = <dyn ToElements<BaseElement>>::to_elements(&program).len();
        let num_assertions = NUM_ASSERTIONS;

        let aux_degrees = memory::get_aux_transition_constraint_degrees();
        let aux_assertions = 2;

        Self {
            context: AirContext::new_multi_segment(
                trace_info,
                degrees,
                aux_degrees,
                num_assertions,
                aux_assertions,
                options,
            )
            .set_num_transition_exemptions(2),
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        memory::get_periodic_column_values()
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField> + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let mut index = 0;
        // cpu
        index += cpu::lui::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::auipc::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::addi::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::jal::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::jalr::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::slti::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        // memory
        index += memory::evaluate_transitions::<E>(frame, periodic_values, &mut result[index..]);
        assert_eq!(index, self.context().num_main_transition_constraints());
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let mut res = Vec::new();
        // for i in 0..NUM_ASSERTIONS {
        res.push(Assertion::single(LOADING, 0, Self::BaseField::ONE));
        // }
        res
    }

    fn evaluate_aux_transition<F, E>(
        &self,
        main_frame: &EvaluationFrame<F>,
        aux_frame: &EvaluationFrame<E>,
        _periodic_values: &[F],
        aux_rand_elements: &AuxTraceRandElements<E>,
        result: &mut [E],
    ) where
        F: FieldElement<BaseField = Self::BaseField>,
        E: FieldElement<BaseField = Self::BaseField> + ExtensionOf<F>,
    {
        // --- memory ----------------------------------------------------------------------
        memory::enforce_aux_constraints::<F, E>(main_frame, aux_frame, aux_rand_elements, result);
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        _aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let mut result: Vec<Assertion<E>> = Vec::new();

        memory::get_aux_assertions_first_step(&mut result);
        let last_step = self.last_step();

        memory::get_aux_assertions_last_step(&mut result, last_step);
        result
    }
}
