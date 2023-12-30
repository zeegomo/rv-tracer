mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use rv_tracer::{prove, verify};
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

generate_tests!(Lui, Rd, Uimm);
generate_tests!(Auipc, Rd, Uimm, Pc);
generate_tests!(Addi, Rd, Rs1, Imm);
generate_tests!(Jal, Rd, Pc);
generate_tests!(Jalr, Rd, Pc, Rs1);
generate_tests!(Slti, Rd, BinRd, Rs1, Imm);
