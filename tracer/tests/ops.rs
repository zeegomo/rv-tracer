mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use trace_defs::TRACE_WIDTH;
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
                    let air = rv_tracer::air::RiscvAir::new(trace_info, (), PROOF_OPTIONS);
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(TRACE_WIDTH);
                    table.read_main_frame(0, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);

                    results == vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                }


                $(
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<BaseElement, $op, $perturb>) -> bool {
                        let table = trace.table;
                        let trace_info = table.get_info();
                        let air = rv_tracer::air::RiscvAir::new(trace_info, (), PROOF_OPTIONS);
                        let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                        let mut frame = EvaluationFrame::new(TRACE_WIDTH);
                        table.read_main_frame(0, &mut frame);
                        air.evaluate_transition(&frame, &[], &mut results);

                        results != vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                    }
                )*

                #[allow(non_snake_case)]
                fn [<test_ $op _conversion>](op: $op) -> bool{
                    let bytes = op.to_op();
                    let parsed = rvsim::Op::parse(bytes).unwrap();
                    let op = rvsim::Op::from(op);
                    println!("{:?} != {:?}", op, parsed);
                    parsed == op
                }
            }
        }
    };
}

generate_tests!(Lui, RdBits, Uimm);
generate_tests!(Auipc, RdBits, Uimm, Pc);
generate_tests!(Addi, RdBits, Rs1Bits, Imm);
generate_tests!(Jal, RdBits, Pc);
generate_tests!(Jalr, RdBits, Pc, Rs1Bits);
generate_tests!(Slti, RdBits, Rs1Bits, Imm);
