mod ops;

use miden_air::constraints::chiplets;
use miden_core::ExtensionOf;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement},
    Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions, TraceInfo,
};
pub type BaseField = winterfell::math::fields::f64::BaseElement;

pub struct RiscvAir {
    context: AirContext<BaseElement>,
}

use trace_defs::{AUX_TRACE_WIDTH, CHIPLETS_START, CHIPLETS_WIDTH, MAIN_TRACE_WIDTH};

impl RiscvAir {
    /// Returns last step of the execution trace.
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for RiscvAir {
    type BaseField = BaseElement;
    type PublicInputs = ();

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, _pub_inputs: (), options: ProofOptions) -> Self {
        assert_eq!(MAIN_TRACE_WIDTH + AUX_TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        degrees.extend(ops::lui::constraint_degrees());
        degrees.extend(ops::auipc::constraint_degrees());
        degrees.extend(ops::addi::constraint_degrees());
        degrees.extend(ops::jal::constraint_degrees());
        degrees.extend(ops::jalr::constraint_degrees());
        degrees.extend(ops::slti::constraint_degrees());

        let mut chiplets_degrees = chiplets::get_transition_constraint_degrees();
        degrees.append(&mut chiplets_degrees);
        // We also need to specify the exact number of assertions we will place against the
        // execution trace. This number must be the same as the number of items in a vector
        // returned from the get_assertions() method below.
        let num_assertions = 1;

        let aux_degrees = chiplets::get_aux_transition_constraint_degrees();
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

    // TODO: remove
    fn get_periodic_column_values(&self) -> Vec<Vec<BaseElement>> {
        chiplets::get_periodic_column_values()
    }

    fn evaluate_transition<E: FieldElement<BaseField = Self::BaseField> + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        periodic_values: &[E],
        result: &mut [E],
    ) {
        let mut index = 0;
        index += ops::lui::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += ops::auipc::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += ops::addi::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += ops::jal::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += ops::jalr::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += ops::slti::evaluate_transitions(frame, periodic_values, &mut result[index..]);

        // println!("index: {}", index);
        // --- chiplets (hasher, bitwise, memory) -------------------------
        chiplets::enforce_constraints::<E>(
            frame,
            periodic_values,
            &mut result[index..index + chiplets::get_transition_constraint_count()],
        );
        index += chiplets::get_transition_constraint_count();
        assert_eq!(index, self.context().num_main_transition_constraints());
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        // all registers should be 0 at the start of the computation
        // let last_step = self.trace_length() - 1;
        vec![Assertion::single(
            trace_defs::LOADING,
            0,
            Self::BaseField::ONE,
        )]
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
        // --- range checker ----------------------------------------------------------------------
        chiplets::enforce_aux_constraints::<F, E>(main_frame, aux_frame, aux_rand_elements, result);
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        _aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let mut result: Vec<Assertion<E>> = Vec::new();

        chiplets::get_aux_assertions_first_step(&mut result);
        let last_step = self.last_step();

        // // Add the range checker's auxiliary column assertions for the last step.
        chiplets::get_aux_assertions_last_step(&mut result, last_step);
        result
    }
}
