mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use quickcheck::Arbitrary;
use quickcheck::TestResult;
use rv_tracer::{
    air::{Inputs, Segment},
    prove, prove_segmented, verify, verify_segmented,
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
                fn [<test_ $op _ok_half_split>](trace: Trace<$op>) -> bool {
                    let row = <Trace<$op>>::op_start();
                    let n_cycles = trace.generate().length() - 1;
                    let split_size = (n_cycles + 1) /  2;
                    let tables = trace.generate_with_splits(split_size as u32);
                    assert_eq!(tables.len(), 2);

                    for table in tables {
                        let inputs = Inputs {
                            program: trace.program(),
                            segment: Segment {
                                segment_n: 0,
                            },
                            n_cycles
                        };

                        let trace_info = table.get_info();
                        let air = rv_tracer::air::RiscvAir::new(trace_info, inputs, PROOF_OPTIONS.clone());
                        let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                        let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                        table.read_main_frame(row, &mut frame);
                        air.evaluate_transition(&frame, &[], &mut results);
                        if results != vec![BaseElement::ZERO; air.context().num_transition_constraints()] {
                            return false;
                        }
                    }
                    true
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
                    let proof = prove::<Blake3_192, BaseElement>(tt, PROOF_OPTIONS.clone(), inputs.clone()).unwrap();
                    verify::<Blake3_192>(proof, inputs).is_ok()

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
generate_tests!(Bne, Rs1Bits, Rs2Bits, H0, Pc);

#[derive(Clone, Copy, Debug)]
struct SplitSize(u32);

impl Arbitrary for SplitSize {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let po2 = core::cmp::max(3, u8::arbitrary(g) % 15);
        SplitSize(1 << po2)
    }
}

quickcheck::quickcheck! {
    #[allow(non_snake_case)]
    fn test_prove_and_verify_splits(trace: Trace<[Add; 16]>, split_size: SplitSize) -> bool {
        let split_size = core::cmp::min(split_size.0, trace.generate().length().next_power_of_two() as u32);
        let traces = trace.generate_with_splits(split_size);
        println!("one: {split_size}");
        let n_cycles = traces.iter().map(|t| t.length() - 1).sum();
        let inputs = Inputs {
            program: trace.program(),
            segment: Segment {
                segment_n: 0,
            },
            n_cycles,
        };

        let  (proofs, link_proofs) = prove_segmented::<Blake3_192, BaseElement>(traces, PROOF_OPTIONS.clone(), inputs.clone()).unwrap();

        // // verify all segment proofs
        // for (segment_n, proof) in proofs.iter().enumerate() {
        //     let inputs = Inputs {
        //         program: trace.program(),
        //         segment: Segment {
        //             segment_n: segment_n as u32,
        //         },
        //         n_cycles,
        //     };
        //     verify::<Blake3_192>(proof.clone(), inputs).unwrap();
        // }

        // // verify all links
        // for (proofs, link_proof) in proofs.windows(2).zip(link_proofs) {
        //     let proof_1 = proofs[0].clone();
        //     let proof_2 = proofs[1].clone();
        //     let inputs = Inputs {
        //         program: trace.program(),
        //         segment: Segment {
        //             segment_n: 0,
        //         },
        //         n_cycles,
        //     };
        //     verify_link::<Blake3_192>(proof_1, proof_2, link_proof, inputs).unwrap();
        // }

        verify_segmented::<Blake3_192>(proofs, link_proofs, inputs).unwrap();
        true
    }


}
