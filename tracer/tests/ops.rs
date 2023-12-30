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
                    println!("1: {:?}", trace.op);
                    let table = trace.table();
                    let trace_info = table.get_info();
                    println!("2");
                    let air = rv_tracer::air::RiscvAir::new(trace_info, (), PROOF_OPTIONS.clone());
                    let mut results = vec![BaseElement::ZERO; air.context().num_transition_constraints()];
                    let mut frame = EvaluationFrame::new(MAIN_TRACE_WIDTH);
                    table.read_main_frame(0, &mut frame);
                    air.evaluate_transition(&frame, &[], &mut results);
                    println!("3");
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
                    let mut bits = [0; 32];
                    for i in 0..32 {
                        bits[i] = ((bytes >> (31 - i)) & 1);
                    }
                    println!("{:?} != {:?} | {:?}", op, parsed, bits);
                    parsed == op
                }

                #[allow(non_snake_case)]
                fn [<test_ $op _prove_and_verify>](trace: Trace<[$op; 16]>) -> bool {
                    // FIX: can't easily stack instructions that modify pc
                    $(
                        if std::any::TypeId::of::<$perturb>() == std::any::TypeId::of::<Pc>() {
                            return true;
                        }
                    )*
                    let proof = prove::<Blake3_192>(trace.table(),PROOF_OPTIONS.clone());
                    verify::<Blake3_192>(proof.unwrap()).is_ok()

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
generate_tests!(Sb, Rs2Bits, Rs1Bits, Imm);

#[test]
fn carlos() {
    let trace = Trace {
        op: [
            Sb {
                rs1: 4,
                rs1_value: 1682788952,
                rs2: 3,
                rs2_value: -1575374927,
                imm: 124,
                mem_value: 0,
            },
            Sb {
                rs1: 1,
                rs1_value: -911964640,
                rs2: 10,
                rs2_value: -1834141632,
                imm: 1456,
                mem_value: 236151644,
            },
            Sb {
                rs1: 27,
                rs1_value: -137822720,
                rs2: 29,
                rs2_value: 581404143,
                imm: -1048,
                mem_value: -1634845149,
            },
            Sb {
                rs1: 31,
                rs1_value: -1068206328,
                rs2: 0,
                rs2_value: 0,
                imm: 0,
                mem_value: 984675301,
            },
            Sb {
                rs1: 7,
                rs1_value: -1278525588,
                rs2: 6,
                rs2_value: 2067810543,
                imm: -1796,
                mem_value: 1866443069,
            },
            Sb {
                rs1: 8,
                rs1_value: -1840430216,
                rs2: 5,
                rs2_value: -746953552,
                imm: 1400,
                mem_value: -1305138025,
            },
            Sb {
                rs1: 10,
                rs1_value: 1099471316,
                rs2: 16,
                rs2_value: 899446992,
                imm: 592,
                mem_value: 1740279965,
            },
            Sb {
                rs1: 8,
                rs1_value: 1594845464,
                rs2: 11,
                rs2_value: 0,
                imm: -808,
                mem_value: 2147483647,
            },
            Sb {
                rs1: 25,
                rs1_value: 1254825052,
                rs2: 23,
                rs2_value: 2121610648,
                imm: 1216,
                mem_value: -891342680,
            },
            Sb {
                rs1: 30,
                rs1_value: -474059884,
                rs2: 27,
                rs2_value: 1578874543,
                imm: 2044,
                mem_value: -1244793229,
            },
            Sb {
                rs1: 16,
                rs1_value: -1122627280,
                rs2: 0,
                rs2_value: 0,
                imm: 552,
                mem_value: -787991299,
            },
            Sb {
                rs1: 6,
                rs1_value: -410491480,
                rs2: 23,
                rs2_value: -319359478,
                imm: 1672,
                mem_value: 1196366823,
            },
            Sb {
                rs1: 30,
                rs1_value: 46779364,
                rs2: 28,
                rs2_value: 1016545863,
                imm: 2044,
                mem_value: -1709629005,
            },
            Sb {
                rs1: 19,
                rs1_value: 264064020,
                rs2: 18,
                rs2_value: 806887461,
                imm: -2048,
                mem_value: -62204507,
            },
            Sb {
                rs1: 31,
                rs1_value: -1134746756,
                rs2: 12,
                rs2_value: 489245241,
                imm: -2048,
                mem_value: -1167602011,
            },
            Sb {
                rs1: 11,
                rs1_value: -263936736,
                rs2: 15,
                rs2_value: -1118391089,
                imm: -2048,
                mem_value: 1538869527,
            },
        ],
    };
    let proof = prove::<Blake3_192>(trace.table(), PROOF_OPTIONS.clone());
    assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
}
