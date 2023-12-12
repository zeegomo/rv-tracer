mod common;
use common::*;
use proptest::prelude::*;
use rv_tracer::{prove, verify};

fn generate_trace() {}

proptest! {
    #[test]
    fn test_lui_ok(state: CpuState, lui: Lui) {
        let mut trace = lui.execute(state);
        let proof = prove::<Blake3_192>(trace,PROOF_OPTIONS);
        prop_assert!(proof.is_ok());
        prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
    }

}
