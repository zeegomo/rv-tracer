// Upper immediate ops: lui, auipc
use crate::trace::*;

use winterfell::math::FieldElement;

macro_rules! ui_op {
    ($name:ident, $flag:expr, $constraints:expr, $const_degree:expr) => {
        pub mod $name {
            use crate::air::{utils::binary_flag, BaseField, FieldElement};
            use crate::trace::*;
            use winterfell::{EvaluationFrame, TransitionConstraintDegree};

            const OP_FLAG: &'static str = $flag;
            const OP_FLAG_DEG: usize = OP_FLAG.len();
            const RD_FLAG_DEG: usize = 5;
            const N_REGISTERS: usize = 1;
            const PAD_FLAG_DEG: usize = 1;


            pub fn evaluate_transitions<E: FieldElement + From<BaseField>>(
                frame: &EvaluationFrame<E>,
                _periodic_values: &[E],
                result: &mut [E],
            ) -> usize {
                let current = frame.current();
                let next = frame.next();
                let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]);
                let op_flag = op_flag * current[BODY];
                
                for rd in 0..N_REGISTERS {
                    let rd_flag = rd_flag(rd as u8, &current[RD_END..RD_END + 5]);
                    result[rd] = ($constraints(current, next, rd)) * rd_flag * op_flag;
                }

                // return the number of use constraint columns
                N_REGISTERS
            }

            pub fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
                // all constraints are of the same degree 6 + 4 + 1
                let mut degrees = vec![];
                for _ in 0..N_REGISTERS {
                    degrees.push(TransitionConstraintDegree::new(
                        RD_FLAG_DEG + OP_FLAG_DEG + PAD_FLAG_DEG + $const_degree,
                    ));
                }
                degrees
            }

            // Degree: 6
            fn op_flag<E>(test: &[E]) -> E
            where
                E: FieldElement,
            {
                assert_eq!(test.len(), 7);
                binary_flag(OP_FLAG, test, E::ONE)
            }

            // Degree: 4
            fn rd_flag<E>(rd: u8, test: &[E]) -> E
            where
                E: FieldElement,
            {
                assert_eq!(test.len(), 5);
                binary_flag(&format!("{rd:05b}"), test, E::ONE)
            }
        }
    };
}

// Degree: 1
fn get_upper_immediate<E: FieldElement>(op: &[E]) -> E {
    let mut result = E::ZERO;
    assert_eq!(op.len(), 20);
    for (i, bit) in op.iter().enumerate() {
        result += *bit * E::from(1u32 << (i + 12));
    }
    result
}

fn op_constraint<E: FieldElement>(current: &[E], next: &[E], rd: usize, offset: E) -> E {
    next[REGISTER_START + rd] - (get_upper_immediate(&current[UIMM_END..UIMM_END + 20]) + offset)
}

ui_op!(
    lui,
    "0110111",
    |current: &[E], next: &[E], rd| { super::op_constraint(current, next, rd, E::ZERO) },
    1
);

ui_op!(
    auipc,
    "0010111",
    |current: &[E], next: &[E], rd| { super::op_constraint(current, next, rd, current[PC]) },
    1
);
