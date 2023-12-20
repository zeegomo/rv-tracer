pub mod ops;
pub mod perturb;

use perturb::Field;
use quickcheck::{Arbitrary, Gen};
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

const OP_ADDR: u32 = 0x200;

pub trait Op: Arbitrary + Debug + Clone {
    fn to_op(&self) -> u32;
    fn execute<E: StarkField>(&self, state: CpuState) -> TraceTable<E>;
}

#[derive(Debug, Clone)]
pub struct CpuState {
    pub regs: [u32; 32],
}

fn load_op_at_addr<O: Op>(addr: u32, op: &O) -> SimpleMemory {
    let mut mem = SimpleMemory::new();
    let op = op.to_op();
    mem.load_slice(addr, &op.to_le_bytes());
    mem
}

#[derive(Debug, Clone)]
pub struct Trace<O: Op> {
    op: O,
}

impl<O: Op> Arbitrary for Trace<O> {
    fn arbitrary(g: &mut Gen) -> Self {
        let op = O::arbitrary(g);
        Self { op }
    }
}

impl<O: Op + Send + 'static> Trace<O> {
    // this is not actually dead
    #[allow(dead_code)]
    pub fn table<E: StarkField + 'static>(&self) -> TraceTable<E> {
        let state = CpuState { regs: [0; 32] };
        self.op.execute(state.clone())
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct PerturbedTrace<E: StarkField, O: Op, P: Field> {
    pub trace_table: TraceTable<E>,
    op: O,
    state: CpuState,
    _phantom: std::marker::PhantomData<P>,
}

pub fn to_binary<const M: usize>(val: u64) -> [u8; M] {
    let mut result = [0; M];
    assert!(
        val < (1u64 << M),
        "requested binary representation of value({val}) bigger than output array({M})"
    );
    for i in 0..M {
        if (val >> i) & 1 == 1 {
            result[M - i - 1] = 1;
        }
    }

    result
}
