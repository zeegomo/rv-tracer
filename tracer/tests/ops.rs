mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use quickcheck::TestResult;
use rv_tracer::{prove, verify};
use std::any::TypeId;
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
                    let row = <Trace<$op>>::op_start();
                    let program = trace.program();
                    let table = trace.table();
                    let trace_info = table.get_info();
                    let air = rv_tracer::air::RiscvAir::new(trace_info, program, PROOF_OPTIONS.clone());
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                    table.read_main_frame(row, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);
                    results == vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                }

                $(
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<$op, $perturb>) -> TestResult {
                        if !trace.op().discard_perturb(TypeId::of::<$perturb>()) {
                            return TestResult::discard();
                        }

                        let row = <PerturbedTrace<$op, $perturb>>::op_start();
                        let program = trace.program();
                        let table = trace.table;
                        let trace_info = table.get_info();

                        let air = rv_tracer::air::RiscvAir::new(trace_info, program, PROOF_OPTIONS.clone());
                        let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                        let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                        table.read_main_frame(row, &mut frame);
                        air.evaluate_transition(&frame, &[], &mut results);
                        TestResult::from_bool(results != vec![BaseElement::ZERO; air.context().num_transition_constraints()])
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
                    let program = trace.program();
                    let proof = prove::<Blake3_192>(trace.table(),PROOF_OPTIONS.clone(), program.clone());
                    verify::<Blake3_192>(proof.unwrap(), program).is_ok()

                }
            }
        }
    };
}

generate_tests!(Lui, RdBits, Uimm);
generate_batched!(Lui);
generate_tests!(Auipc, RdBits, Uimm, Pc);
generate_tests!(Addi, RdBits, Rs1Bits, Imm, H0, H0Bin);
generate_batched!(Addi);
generate_tests!(Jal, RdBits, Pc);
generate_tests!(Jalr, RdBits, Pc, Rs1Bits);
generate_tests!(Slti, RdBits, Rs1Bits, Imm);
generate_batched!(Slti);
generate_tests!(Add, RdBits, Rs1Bits, Rs2Bits, H0, H0Bin);
generate_batched!(Add);
generate_tests!(Bne, Rs1Bits, Rs2Bits, H0);
