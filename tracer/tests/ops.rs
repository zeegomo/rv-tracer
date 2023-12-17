mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use proptest::prelude::*;
use rv_tracer::{prove, verify};
use winterfell::math::fields::f128::BaseElement;

macro_rules! generate_tests {
    ($op:ty, $($perturb:ty),*) => {
        paste::paste! {
            proptest! {



                #[test]
                #[allow(non_snake_case)]
                fn [<test_ $op _ok>](trace: Trace<$op>) {
                    // println!("AAAAAAAAAA1");
                    let proof = prove::<Blake3_192>(trace.table(),PROOF_OPTIONS);
                    // println!("AAAAAAAAAA2");
                    prop_assert!(proof.is_ok());
                    // println!("AAAAAAAAAA3");
                    verify::<Blake3_192>(proof.unwrap()).unwrap();
                    // prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
                    // println!("AAAAAAAAAA4");
                }


                $(
                    #[test]
                    #[should_panic(expected = "did not evaluate to ZERO")]
                    #[allow(non_snake_case)]
                    fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<BaseElement, $op, $perturb>) {
                        // winterfell panics if a constraint does not evaluate to 0 on the trace
                        let _ = prove::<Blake3_192>(trace.trace_table, PROOF_OPTIONS);
                    }
                )*


                #[test]
                #[allow(non_snake_case)]
                fn [<test_ $op _conversion>](op: $op) {
                    let bytes = op.to_op();
                    let parsed = rvsim::Op::parse(bytes).unwrap();
                    prop_assert_eq!($op::from(parsed), op);
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
