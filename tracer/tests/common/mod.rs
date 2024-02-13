pub mod ops;
pub mod perturb;

use once_cell::sync::Lazy;
use perturb::Field;
use quickcheck::{Arbitrary, Gen};
use rv_tracer::{
    air::SegmentConfig,
    executor::{exec, Program},
    trace::TraceTable,
};
use std::{any::TypeId, fmt::Debug};
use trace_defs::MAIN_TRACE_WIDTH;
use winterfell::{
    math::{fields::f64::BaseElement, FieldElement},
    FieldExtension, ProofOptions, Trace as _,
};

const NUM_QUERIES: usize = 10;
const BLOWUP_FACTOR: usize = 16;
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

const OP_ADDR: u32 = 0x20000;

// Generete a RISC-V program that loads the given state into the registers
fn load_state(state: [u32; 32]) -> Vec<u8> {
    state
        .iter()
        .enumerate()
        .flat_map(|(i, &v)| {
            // to load a 32bit value into a register we need to set the upper 20 bits
            // with lui and the lower 12 bits with addi
            let rd = i;
            let u_imm = v & 0xfffff000;
            let i_imm = v & 0xfff;
            ops::Lui {
                rd,
                uimm: u_imm as i32,
            }
            .to_op()
            .to_le_bytes()
            .into_iter()
            .chain(
                ops::Addi {
                    rd,
                    rs1: i,
                    imm: i_imm as i32,
                    rs1_val: 0,
                }
                .to_op()
                .to_le_bytes()
                .into_iter(),
            )
        })
        .collect::<Vec<_>>()
}

macro_rules! to_program {
    ($ops:expr, $state:expr) => {
        Program::new(
            OP_ADDR,
            vec![(
                OP_ADDR,
                load_state($state.regs)
                    .into_iter()
                    .chain(
                        $ops.iter()
                            .flat_map(|o| o.to_op().to_le_bytes().into_iter()),
                    )
                    .collect::<Vec<_>>(),
            )],
        )
    };
    ($ops:expr, $state:expr, ret) => {
        Program::new(
            OP_ADDR,
            vec![(
                OP_ADDR,
                load_state($state.regs)
                    .into_iter()
                    .chain(
                        $ops.iter()
                            .flat_map(|o| o.to_op().to_le_bytes().into_iter()),
                    )
                    .chain(ops::RET.to_op().to_le_bytes().into_iter())
                    .collect::<Vec<_>>(),
            )],
        )
    };
    ($ops:expr, $state:expr, $pc:expr) => {
        Program::new(
            ($pc) - 64 * 4,
            vec![(
                ($pc) - 64 * 4,
                load_state($state.regs)
                    .into_iter()
                    .chain(
                        $ops.iter()
                            .flat_map(|o| o.to_op().to_le_bytes().into_iter()),
                    )
                    .collect::<Vec<_>>(),
            )],
        )
    };
}

pub(crate) use to_program;

pub trait Op: Debug + Clone {
    fn to_op(&self) -> u32;
    fn to_program(&self, state: CpuState) -> Program {
        to_program!(&[self], state)
    }
    fn execute(&self, state: CpuState) -> TraceTable<BaseElement>;
    fn rd(&self) -> u32 {
        let op = self.to_op();
        (op >> 7) & 0x1f
    }
    fn discard_perturb(&self, _perturb_id: TypeId) -> bool {
        false
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
    pub fn generate(&self) -> TraceTable<BaseElement> {
        exec(&self.program(), SegmentConfig::Single).pop().unwrap()
    }

    #[allow(dead_code)]
    pub fn generate_with_splits(&self, segment_len: u32) -> Vec<TraceTable<BaseElement>> {
        exec(&self.program(), SegmentConfig::Split { segment_len })
    }

    #[allow(dead_code)]
    pub fn op_start() -> usize {
        // 64 additional operations are used to load the initial state
        // into registers, and they in turn require 64 addionital cycles
        // to be loaded into memory first
        64 // load the additional operations into memory
        + 1 // load the test operation into memory
        + 64 // execute the additional operations to set registers
    }

    pub fn program(&self) -> Program {
        self.op.to_program(CpuState { regs: [0; 32] })
    }
}

impl<const N: usize, O: Op> Trace<[O; N]> {
    #[allow(dead_code)]
    pub fn generate(&self) -> TraceTable<BaseElement> {
        exec(&self.program(), SegmentConfig::Single).pop().unwrap()
    }

    #[allow(dead_code)]
    pub fn generate_with_splits(&self, segment_len: u32) -> Vec<TraceTable<BaseElement>> {
        exec(&self.program(), SegmentConfig::Split { segment_len })
    }

    pub fn program(&self) -> Program {
        let state: CpuState = CpuState { regs: [0; 32] };
        to_program!(&self.op, state, ret)
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

impl<O: Op, P: Field + 'static> PerturbedTrace<O, P> {
    #[allow(dead_code)]
    pub fn op_start() -> usize {
        // 64 additional operations are used to load the initial state
        // into registers, and they in turn require 64 addionital cycles
        // to be loaded into memory first
        64 // load the additional operations into memory
    + 1 // load the test operation into memory
    + 64 // execute the additional operations to set registers
    }

    #[allow(dead_code)]
    pub fn program(&self) -> Program {
        self.op.to_program(self.state.clone())
    }

    #[allow(dead_code)]
    pub fn op(&self) -> &O {
        &self.op
    }
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
        assert!(
            table.length() > (Self::op_start() + 1).next_power_of_two(),
            "table too short"
        );
        let mut current = [BaseElement::ZERO; MAIN_TRACE_WIDTH];
        let mut next = [BaseElement::ZERO; MAIN_TRACE_WIDTH];
        table.read_row_into(Self::op_start(), &mut current);
        table.read_row_into(Self::op_start() + 1, &mut next);
        P::perturb(&mut current, &mut next, g);
        table.update_row(Self::op_start(), &current);
        table.update_row(Self::op_start() + 1, &next);

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
        table.update_row(Self::op_start(), &self.current);
        table.update_row(Self::op_start() + 1, &self.next);
        Self {
            table,
            op: self.op.clone(),
            state: self.state.clone(),
            current: self.current,
            next: self.next,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<O: Op, P: Field + Clone + 'static> core::fmt::Debug for PerturbedTrace<O, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "op: {:?}, state: {:?}", self.op, self.state)?;
        writeln!(f, "\ncurrent:")?;
        for val in self.current.iter() {
            write!(f, "{} ", val)?;
        }
        writeln!(f, "\nnext:")?;
        for val in self.next.iter() {
            write!(f, "{} ", val)?;
        }
        Ok(())
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
