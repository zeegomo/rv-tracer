mod cpu;
mod memory;

use miden_air::constraints::range;
use winterfell::{
    math::{fields::f64::BaseElement, ExtensionOf, FieldElement, ToElements},
    Air, AirContext, Assertion, AuxTraceRandElements, EvaluationFrame, ProofOptions, TraceInfo,
};
pub type BaseField = winterfell::math::fields::f64::BaseElement;

const MAX_DEG: usize = 16;
const NUM_TRANSITION_EXEMPTIONS: usize = 2;

pub struct RiscvAir {
    context: AirContext<BaseElement>,
    inputs: Inputs,
}

#[derive(Clone, Debug, Copy)]
pub struct Segment {
    // the length of each segment
    pub segment_len: u32,
    // which segment this proof is for
    pub segment_n: u32,
}

#[derive(Clone, Debug)]
pub struct Inputs {
    // the program that was executed
    pub program: Program,
    // the segment of the full execution this proof is for
    pub segment: Segment,
}

impl<E: FieldElement> ToElements<E> for Inputs {
    fn to_elements(&self) -> Vec<E> {
        let mut result = <dyn ToElements<E>>::to_elements(&self.program);
        result.push(E::from(self.segment.segment_len));
        result.push(E::from(self.segment.segment_n));
        result
    }
}

impl Segment {
    fn filter_assertions_for_segment<E: FieldElement>(
        &self,
        assertions: &[Assertion<E>],
    ) -> Vec<Assertion<E>> {
        // the number for usable rows in a segment is one less than the segmet length, because the last
        // fow is used for padding to ensure the degree of the constraints
        let segment_start = self.segment_n * (self.segment_len - 1);
        assertions
            .iter()
            .filter(|assertion| {
                assertion.first_step() >= segment_start as usize
                    && assertion.first_step() < (segment_start + self.segment_len - 1) as usize
            })
            // we need to 'shift' the row of each assertion since this segment first row is actually the segment_start-th row
            // of the complete computation
            .map(|assertion| {
                assert!(assertion.is_single()); // we only support splitting single assertions for now
                Assertion::single(
                    assertion.column(),
                    assertion.first_step() - segment_start as usize,
                    assertion.values()[0],
                )
            })
            .collect()
    }
}

use crate::executor::Program;
use trace_defs::{AUX_DUMMY, AUX_TRACE_WIDTH, BODY, H_3, INSN, LOADING, MAIN_TRACE_WIDTH, PC};

impl RiscvAir {
    /// Returns last step of the execution trace.
    pub fn last_step(&self) -> usize {
        self.trace_length() - self.context().num_transition_exemptions()
    }

    fn get_n_assertions_for_segment(inputs: &Inputs) -> usize {
        // TODO: improve efficiency
        // Winterfell wants at least 1 assertions per segment, in case this segment does not have one from
        // execution constraints, we insert a dummy one
        core::cmp::max(Self::get_assertions_for_segment(inputs).len(), 1)
    }

    fn get_n_aux_assertions_for_segment(inputs: &Inputs, trace_info: &TraceInfo) -> usize {
        // TODO: improve efficiency
        // Winterfell wants at least 1 assertions per segment, in case this segment does not have one from
        // execution constraints, we insert a dummy one
        core::cmp::max(
            Self::get_aux_assertions_for_segment::<BaseElement>(inputs, trace_info).len(),
            1,
        )
    }

    fn get_assertions(inputs: &Inputs) -> Vec<Assertion<BaseElement>> {
        let mut res = Vec::new();
        let mut program_load = <dyn ToElements<BaseElement>>::to_elements(&inputs.program);
        let pc = program_load.remove(0);
        let n_insn = program_load.len();

        res.push(Assertion::single(LOADING, 0, BaseElement::ONE));
        for (i, elem) in program_load.iter().enumerate() {
            // TODO: check we are in the loading phase
            res.push(Assertion::single(INSN, i, *elem));
        }
        res.push(Assertion::single(PC, n_insn, pc));
        // after loading we move to execution
        res.push(Assertion::single(BODY, n_insn, BaseElement::ONE));
        res.push(Assertion::single(H_3, 0, BaseElement::ZERO));

        res
    }

    fn get_aux_assertions<E: FieldElement>(
        inputs: &Inputs,
        trace_info: &TraceInfo,
    ) -> Vec<Assertion<E>> {
        let mut result = Vec::new();

        memory::get_aux_assertions_first_step(&mut result);
        range::get_aux_assertions_first_step(&mut result);

        let last_step = trace_info.length() - NUM_TRANSITION_EXEMPTIONS;
        memory::get_aux_assertions_last_step(&mut result, last_step);
        range::get_aux_assertions_last_step(&mut result, last_step);

        result
    }

    fn get_assertions_for_segment(inputs: &Inputs) -> Vec<Assertion<BaseElement>> {
        let mut assertions = inputs
            .segment
            .filter_assertions_for_segment(&Self::get_assertions(inputs));
        if assertions.is_empty() {
            assertions.push(Assertion::single(H_3, 0, BaseElement::ZERO))
        }
        assertions
    }

    fn get_aux_assertions_for_segment<E: FieldElement>(
        inputs: &Inputs,
        trace_info: &TraceInfo,
    ) -> Vec<Assertion<E>> {
        let mut assertions = inputs
            .segment
            .filter_assertions_for_segment(&Self::get_aux_assertions::<E>(inputs, trace_info));
        if assertions.is_empty() {
            assertions.push(Assertion::single(AUX_DUMMY, 0, E::ZERO))
        }
        assertions
    }
}

impl Air for RiscvAir {
    type BaseField = BaseElement;
    type PublicInputs = Inputs;

    // CONSTRUCTOR
    // --------------------------------------------------------------------------------------------
    fn new(trace_info: TraceInfo, inputs: Inputs, options: ProofOptions) -> Self {
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

        let num_assertions = Self::get_n_assertions_for_segment(&inputs);

        let mut aux_degrees = memory::get_aux_transition_constraint_degrees();
        aux_degrees.extend(range::get_aux_transition_constraint_degrees());
        let aux_assertions = Self::get_n_aux_assertions_for_segment(&inputs);

        Self {
            context: AirContext::new_multi_segment(
                trace_info,
                degrees,
                aux_degrees,
                num_assertions,
                aux_assertions,
                options,
            )
            .set_num_transition_exemptions(NUM_TRANSITION_EXEMPTIONS),
            inputs,
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
        Self::get_assertions_for_segment(&self.inputs)
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
        Self::get_aux_assertions_for_segment(&self.inputs, self.trace_info())
    }
}
