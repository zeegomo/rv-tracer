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

#[test]
fn test_carlos() {
    let trace = execute!(&Lui { rd: 1, uimm: 40000 }, CpuState { regs: [0; 32] });
    let proof = prove::<Blake3_192>(trace, PROOF_OPTIONS);
    verify::<Blake3_192>(proof.unwrap()).unwrap();
}

fn load_op_at_addr<O: Op>(addr: usize, op: &O) -> SimpleMemory {
    let mut mem = SimpleMemory::new();
    let op = op.to_op();
    mem.load_slice(addr as u32, &op.to_le_bytes());
    mem
}
