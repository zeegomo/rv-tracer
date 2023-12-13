mod common;
use common::*;
use proptest::prelude::*;
use rv_tracer::{prove, verify};
use winterfell::math::fields::f128::BaseElement;

// macro_rules! generate_tests! {
//     ($op:ty) => {
//         proptest! {
//             #[test]
//             fn stringify!($op)(state: CpuState, op: $op) {
//                 let trace = op.execute(state);
//                 let proof = prove::<Blake3_192>(trace,PROOF_OPTIONS);
//                 prop_assert!(proof.is_ok());
//                 prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
//             }
//         }
//     }
// }

proptest! {
    #[test]
    fn test_lui_ok(trace: Trace<BaseElement, Lui>) {
        let proof = prove::<Blake3_192>(trace.trace_table,PROOF_OPTIONS);
        prop_assert!(proof.is_ok());
        prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
    }

    #[test]
    #[should_panic]
    fn test_lui_neg(trace: PerturbedTrace<BaseElement, Lui>) {
        // winterfell panics if a constraint does not evaluate to 0 on the trace
        let proof = prove::<Blake3_192>(trace.trace_table, PROOF_OPTIONS);
    }

    // #[test]
    // fn test_auipc_ok(state: CpuState, lui: Lui) {
    //     let trace = lui.execute(state);
    //     let proof = prove::<Blake3_192>(trace,PROOF_OPTIONS);
    //     prop_assert!(proof.is_ok());
    //     prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
    // }

}
