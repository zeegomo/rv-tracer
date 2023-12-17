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
                    let proof = prove::<Blake3_192>(trace.table(),PROOF_OPTIONS);
                    prop_assert!(proof.is_ok());
                    prop_assert!(verify::<Blake3_192>(proof.unwrap()).is_ok());
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

const OP_ADDR: usize = SimpleMemory::DRAM_BASE as usize;
use rv_tracer::sim::{memory::SimpleMemory, Tracer};
macro_rules! execute {
    ($op:expr, $state:expr) => {{
        let mut mem = load_op_at_addr(OP_ADDR, $op);
        let mut cpu_state = rvsim::CpuState::new(OP_ADDR as u32);
        let mut clock = rvsim::SimpleClock::new();
        cpu_state.x = $state.regs;
        let interp = rvsim::Interp::new(&mut cpu_state, &mut mem, &mut clock);
        let tracer = Tracer::new(interp);
        let trace = tracer.build_trace::<BaseElement>();
        trace
    }};
}

// #[test]
// fn test_carlos() {
//     let trace = execute!(
//         &Jalr {
//             rd: 18,
//             rs1: 22,
//             imm: 1716
//         },
//         CpuState {
//             regs: [
//                 0, 103227216, 3903236615, 942388930, 3704867152, 1426223762, 1873744154,
//                 1177832415, 2286646153, 3013512938, 1261124985, 2341075063, 4214371790, 3665819111,
//                 1078958973, 254200516, 444374072, 1016438104, 4274272217, 3844678983, 1564886615,
//                 481745516, 1959246763, 4196546678, 3583885741, 1025997657, 644769290, 1278879976,
//                 1097710302, 602201267, 3681744143, 795865264,
//             ]
//         }
//     );
//     let proof = prove::<Blake3_192>(trace, PROOF_OPTIONS);
//     verify::<Blake3_192>(proof.unwrap()).unwrap();
// }

fn load_op_at_addr<O: Op>(addr: usize, op: &O) -> SimpleMemory {
    let mut mem = SimpleMemory::new();
    let op = op.to_op();
    mem.load_slice(addr as u32, &op.to_le_bytes());
    mem
}
