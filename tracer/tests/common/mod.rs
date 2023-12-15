pub mod ops;
pub mod perturb;

use perturb::Field;
use proptest::prelude::*;
use rv_tracer::sim::{memory::SimpleMemory, Tracer};
use std::fmt::Debug;
use trace_defs::TRACE_WIDTH;
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

pub trait Op: Arbitrary + Debug + From<rvsim::Op> + Eq + Clone {
    fn to_op(&self) -> u32;
    fn execute<E: StarkField>(&self, state: CpuState) -> TraceTable<E>;
}

#[derive(Debug, Clone)]
pub struct CpuState {
    pub regs: [u32; 32],
}

impl Arbitrary for CpuState {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop::collection::vec(i32::MIN..=i32::MAX, 31)
            .prop_map(|regs| CpuState {
                regs: std::iter::once(0)
                    .chain(regs)
                    .map(|x| x as u32)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap(),
            })
            .boxed()
    }
}

fn load_op_at_addr<O: Op>(addr: usize, op: &O) -> SimpleMemory {
    let mut mem = SimpleMemory::new();
    let op = op.to_op();
    mem.load_slice(addr as u32, &op.to_le_bytes());
    mem
}

#[derive(Debug)]
pub struct Trace<O: Op> {
    op: O,
    state: CpuState,
}

impl<O: Op> Arbitrary for Trace<O>
where
    <O as proptest::arbitrary::Arbitrary>::Strategy: 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // TODO: shrinkg only starting state
        (any::<CpuState>(), any::<O>())
            .prop_map(|(state, op)| Trace {
                op: op.clone(),
                state: state.clone(),
            })
            .boxed()
    }
}

impl<O: Op + Send + 'static> Trace<O> {
    // this is not actually dead
    #[allow(dead_code)]
    pub fn table<E: StarkField + 'static>(&self) -> TraceTable<E> {
        // let op = self.op.clone();
        // let state = self.state.clone();
        // let table = std::thread::spawn(move || op.execute(state));
        // std::thread::sleep(std::time::Duration::from_secs(1));
        // if !table.is_finished() {
        //     panic!("{:?}", self);
        // } else {
        //     table.join().unwrap()
        // }
        self.op.execute(self.state.clone())
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

impl<E: StarkField, O: Op, P: Field> Arbitrary for PerturbedTrace<E, O, P>
where
    <O as proptest::arbitrary::Arbitrary>::Strategy: 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // TODO: shrinkg only starting state
        (any::<CpuState>(), any::<O>())
            .prop_map(|(state, op)| PerturbedTrace {
                op: op.clone(),
                state: state.clone(),
                trace_table: op.execute(state),
                _phantom: std::marker::PhantomData,
            })
            .prop_perturb(|mut trace, mut rng| {
                let mut row = [E::ZERO; TRACE_WIDTH];
                trace.trace_table.read_row_into(0, &mut row);

                P::perturb(&mut row, &mut rng);

                trace.trace_table.update_row(0, &row);
                trace
            })
            .boxed()
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
