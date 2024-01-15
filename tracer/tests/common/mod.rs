pub mod ops;
pub mod perturb;

use once_cell::sync::Lazy;
use perturb::Field;
use quickcheck::{Arbitrary, Gen};
use rv_tracer::{
    sim::{LoadData, Tracer},
    trace::TraceTable,
};
use std::fmt::Debug;
use trace_defs::MAIN_TRACE_WIDTH;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement},
    FieldExtension, ProofOptions,
};

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 32;
const GRINDING_FACTOR: u32 = 5;
const FRI_FOLDING_FACTOR: usize = 4;
const FRI_REMAINDER_MAX_DEGREE: usize = 255;

pub type Blake3_192 = winterfell::crypto::hashers::Blake3_192<BaseElement>;

pub static PROOF_OPTIONS: Lazy<ProofOptions> = Lazy::new(|| {
    ProofOptions::new(
        NUM_QUERIES,
        BLOWUP_FACTOR,
        GRINDING_FACTOR,
        FieldExtension::None,
        FRI_FOLDING_FACTOR,
        FRI_REMAINDER_MAX_DEGREE,
    )
});

const OP_ADDR: u32 = 0x200;

macro_rules! execute {
    ($ops:expr, $state:expr) => {{
        let load_data = LoadData::new(vec![(
            OP_ADDR,
            $ops.iter()
                .flat_map(|o| o.to_op().to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        )]);
        let mut cpu_state = rvsim::CpuState::new(OP_ADDR as u32);
        cpu_state.x = $state.regs;
        let tracer = Tracer::new(cpu_state, load_data);
        let trace = tracer.build_trace();
        trace
    }};
    ($ops:expr, $state:expr, $pc:expr) => {{
        let load_data = LoadData::new(vec![(
            $pc,
            $ops.iter()
                .flat_map(|o| o.to_op().to_le_bytes().into_iter())
                .collect::<Vec<_>>(),
        )]);
        let mut cpu_state = rvsim::CpuState::new($pc);
        cpu_state.x = $state.regs;
        let tracer = Tracer::new(cpu_state, load_data);
        let trace = tracer.build_trace();
        trace
    }};
}

pub(crate) use execute;

pub trait Op: Debug + Clone {
    fn to_op(&self) -> u32;
    fn execute(&self, state: CpuState) -> TraceTable<BaseElement>;
    fn rd(&self) -> u32 {
        let op = self.to_op();
        (op >> 7) & 0x1f
    }
}

impl<T: Op> Op for &T {
    fn to_op(&self) -> u32 {
        (*self).to_op()
    }

    fn execute(&self, state: CpuState) -> TraceTable<BaseElement> {
        (*self).execute(state)
    }
}

#[derive(Debug, Clone)]
pub struct CpuState {
    pub regs: [u32; 32],
}

#[derive(Debug, Clone)]
pub struct Trace<O> {
    pub op: O,
}

impl<O: Op + Arbitrary + 'static> Arbitrary for Trace<O> {
    fn arbitrary(g: &mut Gen) -> Self {
        let op = O::arbitrary(g);
        Self { op }
    }
}

impl<const N: usize, O: Op + Arbitrary + 'static> Arbitrary for Trace<[O; N]> {
    fn arbitrary(g: &mut Gen) -> Self {
        let op = (0..N)
            .map(|_| O::arbitrary(g))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self { op }
    }
}

impl<O: Op> Trace<O> {
    // this is not actually dead
    #[allow(dead_code)]
    pub fn table(&self) -> TraceTable<BaseElement> {
        let state: CpuState = CpuState { regs: [0; 32] };
        self.op.execute(state)
    }
}

impl<const N: usize, O: Op> Trace<[O; N]> {
    #[allow(dead_code)]
    pub fn table(&self) -> TraceTable<BaseElement> {
        let state: CpuState = CpuState { regs: [0; 32] };

        execute!(&self.op, state)
    }
}

pub struct PerturbedTrace<O: Op, P: Field> {
    pub table: TraceTable<BaseElement>,
    op: O,
    state: CpuState,
    _phantom: std::marker::PhantomData<P>,
    current: [BaseElement; MAIN_TRACE_WIDTH],
    next: [BaseElement; MAIN_TRACE_WIDTH],
}

impl<O: Op + Arbitrary + 'static, P: Field + Clone + 'static> Arbitrary for PerturbedTrace<O, P> {
    fn arbitrary(g: &mut Gen) -> Self {
        // FIXME: since we don't have constraints for rd = 0 any transition would be valid
        // but we want to generate an invalid one. Remove this once we have constraints for rd = 0
        let mut op = O::arbitrary(g);
        while op.rd() == 0 {
            op = O::arbitrary(g);
        }
        let state = CpuState { regs: [0; 32] };
        let mut table = op.execute(state.clone());

        let mut current = [BaseElement::ZERO; MAIN_TRACE_WIDTH];
        let mut next = [BaseElement::ZERO; MAIN_TRACE_WIDTH];
        table.read_row_into(1, &mut current);
        table.read_row_into(2, &mut next);
        P::perturb(&mut current, &mut next, g);

        table.update_row(1, &current);
        table.update_row(2, &next);
        Self {
            table,
            op,
            state,
            _phantom: std::marker::PhantomData,
            current,
            next,
        }
    }
}

impl<O: Op, P: Field + Clone + 'static> Clone for PerturbedTrace<O, P> {
    fn clone(&self) -> Self {
        let mut table = self.op.execute(self.state.clone());
        table.update_row(0, &self.current);
        table.update_row(1, &self.next);
        Self {
            table,
            op: self.op.clone(),
            state: self.state.clone(),
            current: self.current.clone(),
            next: self.next.clone(),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<O: Op, P: Field + Clone + 'static> core::fmt::Debug for PerturbedTrace<O, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "op: {:?}, state: {:?}, current: {:?}, next: {:?}",
            self.op, self.state, self.current, self.next
        )
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
