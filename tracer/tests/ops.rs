mod common;
use common::ops::*;
use common::perturb::*;
use common::*;
use rv_tracer::{prove, verify};

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


                // $(
                //     #[test]
                //     #[should_panic(expected = "did not evaluate to ZERO")]
                //     #[allow(non_snake_case)]
                //     fn [<test_ $op _ $perturb _neg>](trace: PerturbedTrace<BaseElement, $op, $perturb>) {
                //         // winterfell panics if a constraint does not evaluate to 0 on the trace
                //         let _ = prove::<Blake3_192>(trace.trace_table, PROOF_OPTIONS);
                //     }
                // )*


                #[allow(non_snake_case)]
                fn [<test_ $op _conversion>](op: $op) -> bool{
                    let bytes = op.to_op();
                    let parsed = rvsim::Op::parse(bytes).unwrap();
                    parsed == rvsim::Op::from(op)
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
generate_tests!(Slti, Rd, Rs1, Imm);
