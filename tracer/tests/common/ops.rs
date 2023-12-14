use super::perturb::Field;
use super::*;
use proptest::prelude::*;

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

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Lui {
    rd: usize,
    uimm: i32,
}

impl Arbitrary for Lui {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..(1u32 << 20))
            .prop_map(|(rd, uimm)| Lui {
                rd,
                uimm: (uimm << 12) as i32,
            })
            .boxed()
    }
}

impl Op for Lui {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let uimm = (self.uimm as u32) & 0xfffff000;
        let rd = (self.rd << 7) as u32;
        0b0110111 | uimm | rd
    }

    fn perturb() -> Vec<perturb::Field> {
        vec![Field::Rd, Field::Uimm, Field::RdBits]
    }
}

impl From<rvsim::Op> for Lui {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Lui { rd, u_imm } => Self { rd, uimm: u_imm },
            _ => panic!("wrong op type"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Auipc {
    rd: usize,
    uimm: i32,
}

impl Arbitrary for Auipc {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..(1u32 << 20))
            .prop_map(|(rd, uimm)| Auipc {
                rd,
                uimm: (uimm << 12) as i32,
            })
            .boxed()
    }
}

impl Op for Auipc {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let uimm = (self.uimm as u32) & 0xfffff000;
        let rd = (self.rd << 7) as u32;
        0b0010111 | uimm | rd
    }

    fn perturb() -> Vec<perturb::Field> {
        vec![Field::Rd, Field::Uimm, Field::RdBits, Field::Pc]
    }
}

impl From<rvsim::Op> for Auipc {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Auipc { rd, u_imm } => Self { rd, uimm: u_imm },
            _ => panic!("wrong op type"),
        }
    }
}
