pub mod ops;
pub mod perturb;

use perturb::Field;
use quickcheck::{Arbitrary, Gen};
use rv_tracer::sim::{memory::SimpleMemory, Tracer};
use std::fmt::Debug;
use trace_defs::TRACE_WIDTH;
use winterfell::{
    math::{fields::f64::BaseElement, StarkField},
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
    fn rd(&self) -> u32 {
        let op = self.to_op();
        (op >> 7) & 0x1f
    }
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

impl<O: Op> Trace<O> {
    // this is not actually dead
    #[allow(dead_code)]
    pub fn table<E: StarkField + 'static>(&self) -> TraceTable<E> {
        let state: CpuState = CpuState { regs: [0; 32] };
        self.op.execute(state)
    }
}

#[derive(Debug, Clone)]
pub struct PerturbedTrace<E: StarkField, O: Op, P: Field> {
    pub table: TraceTable<E>,
    _op: O,
    _state: CpuState,
    _phantom: std::marker::PhantomData<P>,
}

impl<E: StarkField + 'static, O: Op, P: Field + Clone + 'static> Arbitrary
    for PerturbedTrace<E, O, P>
{
    fn arbitrary(g: &mut Gen) -> Self {
        // FIXME: since we don't have constraints for rd = 0 any transition would be valid
        // but we want to generate an invalid one. Remove this once we have constraints for rd = 0
        let mut op = O::arbitrary(g);
        while op.rd() == 0 {
            op = O::arbitrary(g);
        }
        let state = CpuState { regs: [0; 32] };
        let mut table = op.execute(state.clone());

        let mut current = [E::ZERO; TRACE_WIDTH];
        let mut next = [E::ZERO; TRACE_WIDTH];
        table.read_row_into(0, &mut current);
        table.read_row_into(1, &mut next);
        P::perturb(&mut current, &mut next, g);

        table.update_row(0, &current);
        table.update_row(1, &next);
        Self {
            table,
            _op: op,
            _state: state,
            _phantom: std::marker::PhantomData,
        }
    }
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
