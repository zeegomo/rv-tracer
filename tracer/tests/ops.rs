mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use rv_tracer::{prove, verify};
use trace_defs::MAIN_TRACE_WIDTH;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement},
    Air, EvaluationFrame, Trace as _,
};

macro_rules! generate_tests {
    ($op:ty, $($perturb:ty),*) => {
        paste::paste! {
            quickcheck::quickcheck! {
                #[allow(non_snake_case)]
                fn [<test_ $op _ok>](trace: Trace<$op>) -> bool {
                    let table = trace.table();
                    let trace_info = table.get_info();
                    let air = rv_tracer::air::RiscvAir::new(trace_info, (), PROOF_OPTIONS.clone());
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                    table.read_main_frame(1, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);
                    results == vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                }


                $(
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<$op, $perturb>) -> bool {
                        let table = trace.table;
                        let trace_info = table.get_info();
                        let air = rv_tracer::air::RiscvAir::new(trace_info, (), PROOF_OPTIONS.clone());
                        let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                        let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                        // the first step is for loading
                        println!("perturb: {:?}", table.get_info());
                        table.read_main_frame(1, &mut frame);
                        air.evaluate_transition(&frame, &[], &mut results);

                        results != vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                    }
                )*

                #[allow(non_snake_case)]
                fn [<test_ $op _conversion>](op: $op) -> bool{
                    let bytes = op.to_op();
                    let parsed = rvsim::Op::parse(bytes).unwrap();
                    let op = rvsim::Op::from(op);
                    let mut bits = [0; 32];
                    for i in 0..32 {
                        bits[i] = ((bytes >> (31 - i)) & 1);
                    }
                    println!("{:?} != {:?} | {:?}", op, parsed, bits);
                    parsed == op
                }
            }
        }
    };
}

macro_rules! generate_batched {
    ($op:ty) => {
        paste::paste! {
            quickcheck::quickcheck! {
                #[allow(non_snake_case)]
                fn [<test_ $op _prove_and_verify>](trace: Trace<[$op; 16]>) -> bool {
                    let proof = prove::<Blake3_192>(trace.table(),PROOF_OPTIONS.clone());
                    verify::<Blake3_192>(proof.unwrap()).is_ok()

                }
            }
        }
    };
}

generate_tests!(Lui, RdBits, Uimm);
generate_batched!(Lui);
generate_tests!(Auipc, RdBits, Uimm, Pc);
generate_tests!(Addi, RdBits, Rs1Bits, Imm);
generate_batched!(Addi);
generate_tests!(Jal, RdBits, Pc);
generate_tests!(Jalr, RdBits, Pc, Rs1Bits);
generate_tests!(Slti, RdBits, Rs1Bits, Imm);
generate_batched!(Slti);

#[test]
fn carlos() {
    let trace = Trace {
        op: [
            Addi {
                rd: 8,
                rs1: 17,
                rs1_val: 1656034299,
                imm: 49,
            },
            Addi {
                rd: 10,
                rs1: 27,
                rs1_val: -2021370797,
                imm: 734,
            },
            Addi {
                rd: 30,
                rs1: 22,
                rs1_val: 615237333,
                imm: 168,
            },
            Addi {
                rd: 23,
                rs1: 26,
                rs1_val: -1470919194,
                imm: -1659,
            },
            Addi {
                rd: 28,
                rs1: 2,
                rs1_val: 1164001425,
                imm: -2048,
            },
            Addi {
                rd: 27,
                rs1: 2,
                rs1_val: -82742876,
                imm: 530,
            },
            Addi {
                rd: 0,
                rs1: 3,
                rs1_val: 1586036761,
                imm: 1101,
            },
            Addi {
                rd: 24,
                rs1: 7,
                rs1_val: 2053606410,
                imm: -154,
            },
            Addi {
                rd: 28,
                rs1: 11,
                rs1_val: -1078523207,
                imm: -1819,
            },
            Addi {
                rd: 18,
                rs1: 29,
                rs1_val: 1616541199,
                imm: 2047,
            },
            Addi {
                rd: 3,
                rs1: 30,
                rs1_val: -1397144209,
                imm: -1028,
            },
            Addi {
                rd: 23,
                rs1: 13,
                rs1_val: 154806733,
                imm: 526,
            },
            Addi {
                rd: 17,
                rs1: 3,
                rs1_val: -2125874979,
                imm: 493,
            },
            Addi {
                rd: 9,
                rs1: 1,
                rs1_val: 1375066508,
                imm: -215,
            },
            Addi {
                rd: 15,
                rs1: 27,
                rs1_val: -209546870,
                imm: -405,
            },
            Addi {
                rd: 24,
                rs1: 13,
                rs1_val: 218720441,
                imm: 855,
            },
        ],
    };

    let proof = prove::<Blake3_192>(trace.table(), PROOF_OPTIONS.clone());
    verify::<Blake3_192>(proof.unwrap()).is_ok();
}
