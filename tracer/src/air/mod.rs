mod cpu;
mod memory;

use miden_air::constraints::range;
use winterfell::{
    math::{fields::f64::BaseElement, ExtensionOf, FieldElement, ToElements},
    Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions, TraceInfo,
};
pub type BaseField = winterfell::math::fields::f64::BaseElement;

const MAX_DEG: usize = 16;

pub struct RiscvAir {
    context: AirContext<BaseElement>,
    program: Program,
}

use crate::executor::Program;
use trace_defs::{AUX_TRACE_WIDTH, BODY, H_3, INSN, LOADING, MAIN_TRACE_WIDTH, PC};

impl RiscvAir {
    /// Returns last step of the execution trace.
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }
}

impl Air for RiscvAir {
    type BaseField = BaseElement;
    type PublicInputs = Program;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, program: Program, options: ProofOptions) -> Self {
        assert_eq!(MAIN_TRACE_WIDTH + AUX_TRACE_WIDTH, trace_info.width());

        let mut degrees = Vec::new();
        degrees.extend(cpu::lui::constraint_degrees());
        degrees.extend(cpu::auipc::constraint_degrees());
        degrees.extend(cpu::addi::constraint_degrees());
        degrees.extend(cpu::jal::constraint_degrees());
        degrees.extend(cpu::jalr::constraint_degrees());
        degrees.extend(cpu::slti::constraint_degrees());
        degrees.extend(cpu::add::constraint_degrees());
        degrees.extend(cpu::bne::constraint_degrees());

        degrees.extend(memory::get_transition_constraint_degrees());
        degrees.extend(range::get_transition_constraint_degrees());

        for (i, degree) in degrees.iter().enumerate() {
            debug_assert!(
                degree.min_blowup_factor() <= MAX_DEG,
                "{i}-th degree {:?} is too large",
                degree
            );
        }
        // One assertion for each instruction of the program binary + 1 for the initial pc value + 2
        // to control the start of the loading and execution phases.
        // let num_assertions = <dyn ToElements<BaseElement>>::to_elements(&program).len() + 2;
        let num_assertions = 1;

        let mut aux_degrees = memory::get_aux_transition_constraint_degrees();
        aux_degrees.extend(range::get_aux_transition_constraint_degrees());
        // let aux_assertions = memory::NUM_AUX_ASSERTIONS + range::NUM_AUX_ASSERTIONS;
        let aux_assertions = 1;

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
            program,
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
        index += cpu::add::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        index += cpu::bne::evaluate_transitions(frame, periodic_values, &mut result[index..]);
        // memory
        index += memory::evaluate_transitions::<E>(frame, periodic_values, &mut result[index..]);
        // range check
        range::enforce_constraints(frame, &mut result[index..]);
        index += range::get_transition_constraint_count();
        assert_eq!(index, self.context().num_main_transition_constraints());
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let mut res = Vec::with_capacity(self.context().num_assertions());
        // let mut program_load = <dyn ToElements<BaseElement>>::to_elements(&self.program);
        // let pc = program_load.remove(0);
        // let n_insn = program_load.len();

        // res.push(Assertion::single(LOADING, 0, BaseElement::ONE));
        // for (i, elem) in program_load.iter().enumerate() {
        //     // TODO: check we are in the loading phase
        //     res.push(Assertion::single(INSN, i, *elem));
        // }
        // res.push(Assertion::single(PC, n_insn, pc));
        // // after loading we move to execution
        // res.push(Assertion::single(BODY, n_insn, BaseElement::ONE));
        res.push(Assertion::single(H_3, 0, BaseElement::ZERO));
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
        range::enforce_aux_constraints(main_frame, aux_frame, aux_rand_elements, &mut result[1..]);
    }

    fn get_aux_assertions<E: FieldElement<BaseField = Self::BaseField>>(
        &self,
        _aux_rand_elements: &AuxTraceRandElements<E>,
    ) -> Vec<Assertion<E>> {
        let mut result: Vec<Assertion<E>> = Vec::new();

        memory::get_aux_assertions_first_step(&mut result);
        range::get_aux_assertions_first_step(&mut result);

        let last_step = self.last_step();
        memory::get_aux_assertions_last_step(&mut result, last_step);
        range::get_aux_assertions_last_step(&mut result, last_step);

        result
    }
}
