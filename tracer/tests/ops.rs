mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use rv_tracer::{prove, verify};
use winterfell::math::fields::f128::BaseElement;

macro_rules! generate_tests {
    ($op:ty, $($perturb:ty),*) => {
        paste::paste! {
            quickcheck::quickcheck! {
                #[allow(non_snake_case)]
                fn [<test_ $op _ok>](trace: Trace<$op>) -> bool {
                    let proof = prove::<Blake3_192>(trace.table(), PROOF_OPTIONS);
                    assert!(proof.is_ok());
                    verify::<Blake3_192>(proof.unwrap()).is_ok()
                }


                $(
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<BaseElement, $op, $perturb>) -> bool {
                        // winterfell panics if a constraint does not evaluate to 0 on the trace
                        match std::panic::catch_unwind(|| { let _ = prove::<Blake3_192>(trace.trace_table, PROOF_OPTIONS); }) {
                            Err(msg) => {
                                if let Some(msg) = msg.downcast_ref::<&'static str>() {
                                    msg.contains("did not evaluate to ZERO") || msg.contains("constraint evaluation failed")
                                } else if let Some(msg) = msg.downcast_ref::<String>() {
                                    msg.contains("did not evaluate to ZERO")
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }
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
