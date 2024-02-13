mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use quickcheck::Arbitrary;
use quickcheck::TestResult;
use rv_tracer::{
    air::{Inputs, Segment},
    prove, verify,
};
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
                    let table = trace.generate();
                    let inputs = Inputs {
                        program: trace.program(),
                        segment: Segment {
                            segment_n: 0,
                        },
                        n_cycles: table.length() - 1,
                    };

                    let trace_info = table.get_info();
                    let air = rv_tracer::air::RiscvAir::new(trace_info, inputs, PROOF_OPTIONS.clone());
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                    table.read_main_frame(row, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);
                    results == vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                }

                #[allow(non_snake_case)]
                fn [<test_ $op _ok_split>](trace: Trace<$op>) -> bool {
                    let row = <Trace<$op>>::op_start();
                    let table = trace.generate();
                    let inputs = Inputs {
                        program: trace.program(),
                        segment: Segment {
                            segment_n: 0,
                        },
                        n_cycles: table.length() - 1,
                    };

                    let trace_info = table.get_info();
                    let air = rv_tracer::air::RiscvAir::new(trace_info, inputs, PROOF_OPTIONS.clone());
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                    table.read_main_frame(row, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);
                    results == vec![BaseElement::ZERO; air.context().num_transition_constraints()]
                }



                $(
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<$op, $perturb>) -> TestResult {
                        if trace.op().discard_perturb(TypeId::of::<$perturb>()) {
                            return TestResult::discard();
                        }

                        let row = <PerturbedTrace<$op, $perturb>>::op_start();
                        let inputs = Inputs {
                            program: trace.program(),
                            segment: Segment {
                                segment_n: 0,
                            },
                            n_cycles: trace.table.length() - 1,
                        };
                        let table = trace.table;
                        let trace_info = table.get_info();

                        let air = rv_tracer::air::RiscvAir::new(trace_info, inputs, PROOF_OPTIONS.clone());
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
                    let tt = trace.generate();
                    let inputs = Inputs {
                        program: trace.program(),
                        segment: Segment {
                            segment_n: 0,
                        },
                        n_cycles: tt.length() - 1,
                    };

                    let trace_length = tt.length();
                    assert!(trace_length > 16);
                    let proof = prove::<Blake3_192>(tt, PROOF_OPTIONS.clone(), inputs.clone()).unwrap();
                    verify::<Blake3_192>(proof, inputs).is_ok()

                }
            }
        }
    };
}

macro_rules! generate_batched_splits {
    ($op:ty) => {
        paste::paste! {
            quickcheck::quickcheck! {
                #[allow(non_snake_case)]
                fn [<test_ $op _prove_and_verify_splits>](trace: Trace<[$op; 16]>, split_size: SplitSize) -> bool {
                    let traces = trace.generate_with_splits(split_size.0);
                    let n_cycles = traces.iter().map(|t| t.length() - 1).sum();
                    println!("traces: {:?}", traces.len());

                    for (i, segment) in traces.into_iter().enumerate() {
                        let inputs = Inputs {
                            program: trace.program(),
                            segment: Segment {
                                segment_n: i as u32,
                            },
                            n_cycles,
                        };
                        let proof = prove::<Blake3_192>(segment,PROOF_OPTIONS.clone(), inputs.clone()).unwrap();
                        if !verify::<Blake3_192>(proof, inputs).is_ok() {
                            return false;
                        }
                    }

                    true
                }
            }
        }
    };
}

generate_tests!(Lui, RdBits, Uimm);
generate_batched!(Lui);
generate_batched_splits!(Lui);
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

#[derive(Clone, Copy, Debug)]
struct SplitSize(u32);

impl Arbitrary for SplitSize {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let po2 = core::cmp::max(3, u8::arbitrary(g) % 15);
        SplitSize(1 << po2)
    }
}
