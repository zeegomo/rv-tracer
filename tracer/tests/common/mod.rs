use proptest::prelude::*;
use rv_tracer::sim::{memory::SimpleMemory, Tracer};
use std::fmt::Debug;
use winterfell::{
    math::{fields::f128::BaseElement, StarkField},
    FieldExtension, ProofOptions, TraceTable,
};

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

pub const PROOF_OPTIONS: ProofOptions = ProofOptions::new(
    NUM_QUERIES,
    BLOWUP_FACTOR,
    GRINDING_FACTOR,
    FieldExtension::None,
    FRI_FOLDING_FACTOR,
    FRI_REMAINDER_MAX_DEGREE,
);

const OP_ADDR: usize = SimpleMemory::DRAM_BASE as usize;

pub trait Op: Debug {
    fn to_op(&self) -> u32;
    fn execute<E: StarkField>(&self, state: CpuState) -> TraceTable<E>;
}

#[derive(Debug)]
pub struct CpuState {
    pub regs: [u32; 32],
}

impl Arbitrary for CpuState {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop::collection::vec(0..u32::MAX, 31)
            .prop_map(|regs| CpuState {
                regs: std::iter::once(0)
                    .chain(regs.into_iter())
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            })
            .boxed()
    }
}

#[derive(Debug)]
pub struct Lui {
    rd: usize,
    imm: u32,
}

impl Arbitrary for Lui {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..(1u32 << 20))
            .prop_map(|(rd, imm)| Lui { rd, imm })
            .boxed()
    }
}

fn load_op_at_addr<O: Op>(addr: usize, op: &O) -> SimpleMemory {
    let mut mem = SimpleMemory::new();
    let op = op.to_op();
    mem.load_slice(addr as u32, &op.to_le_bytes());
    mem
}

impl Op for Lui {
    fn execute<E>(&self, mut state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        let mut mem = load_op_at_addr(OP_ADDR, self);
        let mut cpu_state = rvsim::CpuState::new(OP_ADDR as u32);
        let mut clock = rvsim::SimpleClock::new();
        cpu_state.x = state.regs;
        let interp = rvsim::Interp::new(&mut cpu_state, &mut mem, &mut clock);
        let tracer = Tracer::new(interp);
        let trace = tracer.build_trace();
        println!("state: {:?}", state);
        trace
    }

    fn to_op(&self) -> u32 {
        let imm = self.imm << 12;
        let rd = (self.rd << 7) as u32;
        0b0110111 | imm | rd
    }
}
