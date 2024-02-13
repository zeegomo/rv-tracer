use miden_air::constraints::chiplets;
use winterfell::{
    math::{fields::f64::BaseElement, ExtensionOf, FieldElement},
    Assertion, AuxTraceRandElements, EvaluationFrame, TransitionConstraintDegree,
};

/// Returns the set of periodic columns required by chiplets in the Chiplets module.
pub fn get_periodic_column_values() -> Vec<Vec<BaseElement>> {
    chiplets::get_periodic_column_values()
}

/// Returns the range checker's boundary assertions for the main trace at the first step.
pub fn get_aux_assertions_first_step<E: FieldElement>(result: &mut Vec<Assertion<E>>) {
    chiplets::get_aux_assertions_first_step(result);
}

/// Returns the range checker's boundary assertions for the main trace at the last step.
pub fn get_aux_assertions_last_step<E: FieldElement>(result: &mut Vec<Assertion<E>>, step: usize) {
    chiplets::get_aux_assertions_last_step(result, step);
}

// CHIPLETS TRANSITION CONSTRAINTS
// ================================================================================================

/// Builds the transition constraint degrees for the chiplets module and all chiplet components.
pub fn get_transition_constraint_degrees() -> Vec<TransitionConstraintDegree> {
    chiplets::get_transition_constraint_degrees()
}

/// Returns the number of transition constraints for the chiplets.
pub fn get_transition_constraint_count() -> usize {
    chiplets::get_transition_constraint_count()
}

/// Enforces constraints for the chiplets module and all chiplet components.
pub fn evaluate_transitions<E: FieldElement<BaseField = BaseElement>>(
    frame: &EvaluationFrame<E>,
    periodic_values: &[E],
    result: &mut [E],
) -> usize {
    chiplets::enforce_constraints(frame, periodic_values, result);
    get_transition_constraint_count()
}

/// Returns the transition constraint degrees for the range checker's auxiliary columns, used for
/// multiset checks.
pub fn get_aux_transition_constraint_degrees() -> Vec<TransitionConstraintDegree> {
    chiplets::get_aux_transition_constraint_degrees()
}

/// Enforces constraints on the range checker's auxiliary columns.
pub fn enforce_aux_constraints<F, E>(
    main_frame: &EvaluationFrame<F>,
    aux_frame: &EvaluationFrame<E>,
    aux_rand_elements: &AuxTraceRandElements<E>,
    result: &mut [E],
) where
    F: FieldElement<BaseField = BaseElement>,
    E: FieldElement<BaseField = BaseElement> + ExtensionOf<F>,
{
    chiplets::enforce_aux_constraints::<F, E>(main_frame, aux_frame, aux_rand_elements, result)
}
