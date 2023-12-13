mod perturb;

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

macro_rules! execute {
    ($op:expr, $state:expr) => {{
        let mut mem = load_op_at_addr(OP_ADDR, $op);
        let mut cpu_state = rvsim::CpuState::new(OP_ADDR as u32);
        let mut clock = rvsim::SimpleClock::new();
        cpu_state.x = $state.regs;
        let interp = rvsim::Interp::new(&mut cpu_state, &mut mem, &mut clock);
        let tracer = Tracer::new(interp);
        let trace = tracer.build_trace();
        trace
    }};
}

pub trait Op: Arbitrary + Debug {
    fn to_op(&self) -> u32;
    fn execute<E: StarkField>(&self, state: CpuState) -> TraceTable<E>;
    fn perturb() -> Vec<perturb::Field>;
}

pub trait Perturbation {
    fn perturb<E: StarkField>(trace: &mut TraceTable<E>);
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
                    .chain(regs)
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
    uimm: u32,
}

impl Arbitrary for Lui {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..(1u32 << 20))
            .prop_map(|(rd, uimm)| Lui { rd, uimm })
            .boxed()
    }
}

#[derive(Debug)]
pub struct Auipc {
    rd: usize,
    uimm: u32,
}

impl Arbitrary for Auipc {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..(1u32 << 20))
            .prop_map(|(rd, uimm)| Auipc { rd, uimm })
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
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let uimm = self.uimm << 12;
        let rd = (self.rd << 7) as u32;
        0b0110111 | uimm | rd
    }

    fn perturb() -> Vec<perturb::Field> {
        vec![perturb::Field::Rd, perturb::Field::Uimm]
    }
}

// impl Op for Auipc {
//     fn execute<E>(&self, state: CpuState) -> TraceTable<E>
//     where
//         E: StarkField,
//     {
//         execute!(self, state)
//     }

//     fn to_op(&self) -> u32 {
//         let uimm = self.uimm << 12;
//         let rd = (self.rd << 7) as u32;
//         0b0010111 | uimm | rd
//     }
// }

#[derive(Debug)]
pub struct Trace<E: StarkField, O: Op> {
    pub trace_table: TraceTable<E>,
    _phantom: std::marker::PhantomData<O>,
}

impl<E: StarkField, O: Op> Arbitrary for Trace<E, O>
where
    <O as proptest::arbitrary::Arbitrary>::Strategy: 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // TODO: shrinkg only starting state
        (any::<CpuState>(), any::<O>())
            .prop_map(|(state, op)| Trace {
                trace_table: op.execute(state),
                _phantom: std::marker::PhantomData,
            })
            .boxed()
    }
}

#[derive(Debug)]
pub struct PerturbedTrace<E: StarkField, O: Op> {
    pub trace_table: TraceTable<E>,
    _phantom: std::marker::PhantomData<O>,
}

impl<E: StarkField, O: Op> Arbitrary for PerturbedTrace<E, O>
where
    <O as proptest::arbitrary::Arbitrary>::Strategy: 'static,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // TODO: shrinkg only starting state
        (any::<CpuState>(), any::<O>())
            .prop_map(|(state, op)| PerturbedTrace {
                trace_table: op.execute(state),
                _phantom: std::marker::PhantomData,
            })
            .prop_perturb(|mut trace, mut rng| {
                let mut row = [E::ZERO; TRACE_WIDTH];
                trace.trace_table.read_row_into(0, &mut row);
                for field in O::perturb() {
                    field.perturb(&mut row, &mut rng);
                }
                trace.trace_table.update_row(0, &row);
                trace
            })
            .boxed()
    }
}
