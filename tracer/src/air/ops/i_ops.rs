// Upper immediate ops: lui, auipc
use crate::trace::*;

use winterfell::math::FieldElement;

macro_rules! i_op {
    (name = $name:ident, opcode = $flag:expr, funct3 = $funct3:expr, $($constraints:expr => deg = $deg:expr),*) => {
        pub mod $name {
            use crate::air::{utils::binary_flag, BaseField, FieldElement};
            use crate::trace::*;
            use winterfell::{EvaluationFrame, TransitionConstraintDegree};

            const OP_FLAG: &'static str = $flag;
            const OP_FLAG_DEG: usize = OP_FLAG.len();
            const REG_FLAG_DEG: usize = 5;
            const FUNCT3_FLAG: &'static str = $disc;
            const FUNCT3_FLAG_DEG: usize = FUNCT3_FLAG.len(); 
            const N_REGISTERS: usize = 1;
            const PAD_FLAG_DEG: usize = 1;


            pub fn evaluate_transitions<E: FieldElement + From<BaseField>>(
                frame: &EvaluationFrame<E>,
                _periodic_values: &[E],
                result: &mut [E],
            ) -> usize {
                let current = frame.current();
                let next = frame.next();
                let op_flag = op_flag(&current[OPCODE_END..OPCODE_END + 7]) * current[BODY] * funct3_flag(&current[FUNCT3_END..FUNCT3_END + 3]);
                
                for rs1 in 0..N_REGISTERS {
                    for rd in 0..N_REGISTERS {
                        let rd_flag = reg_flag(rd as u8, &current[RD_END..RD_END + 5]);
                        let rs1_flag = reg_flag(rs1 as u8, &current[RS1_END..RS1_END + 5]);
                        result[rd] = ($constraints(current, next, rd, rs1)) * rd_flag * rs1_flag * op_flag;
                    }
                }
                
                // return the number of use constraint columns
                N_REGISTERS * N_REGISTERS
            }

            pub fn constraint_degrees() -> Vec<TransitionConstraintDegree> {
                // all constraints are of the same degree 6 + 4 + 1
                let mut degrees = vec![];
                for _ in 0..N_REGISTERS * N_REGISTERS {
                    degrees.push(TransitionConstraintDegree::new(
                        REG_FLAG_DEG + REG_FLAG_DEG + OP_FLAG_DEG + PAD_FLAG_DEG + $const_degree,
                    ));
                }
                degrees
            }

            fn funct3_flag<E>(test: &[E]) -> E
            where
                E: FieldElement,
            {
                assert_eq!(test.len(), 3);
                binary_flag(FUNCT3_FLAG, test, E::ONE)
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
            fn reg_flag<E>(reg: u8, test: &[E]) -> E
            where
                E: FieldElement,
            {
                assert_eq!(test.len(), 5);
                binary_flag(&format!("{reg:05b}"), test, E::ONE)
            }
        }
    };
}

// Degree: 1
// TODO: this is unsigned, but we need signed
fn get_immediate<E: FieldElement>(op: &[E]) -> E {

    let mut result = E::ZERO;
    assert_eq!(op.len(), 12);
    for (i, bit) in op.iter().enumerate() {
        result += *bit * E::from(1u32 << i);
    }
    result
}

fn op_constraint<E: FieldElement>(current: &[E], next: &[E], rd: usize, offset: E) -> E {
    next[REGISTER_START + rd] - (get_immediate(&current[IMM_END..IMM_END + 12]) + offset)
}

i_op!(
    name = addi,
    opcode = "0010011",
    funct3 = "000",
    super::op_constraint(current, next, rd, current[rs1]) => deg  = 1,
);


i_op!(
    name = sltui,
    opcode = "0010011",
    funct3 = "010",
    next[rd] * current[H_0] + current[rs1] - get_immediate(current) => 2,
    |current: &[E], next: &[E], rd, rs1| { (E::ONE - next[rd]) * current[H_1] + current[rs1] - get_immediate(current) } => 2,
);
