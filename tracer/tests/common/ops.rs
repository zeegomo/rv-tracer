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
    pub rd: usize,
    pub uimm: i32,
}

impl Arbitrary for Lui {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, any::<Signed<20, 12>>())
            .prop_map(|(rd, uimm)| Lui {
                rd,
                uimm: uimm.into(),
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
        (0..32usize, any::<Signed<20, 12>>())
            .prop_map(|(rd, uimm)| Auipc {
                rd,
                uimm: uimm.into(),
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
}

impl From<rvsim::Op> for Auipc {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Auipc { rd, u_imm } => Self { rd, uimm: u_imm },
            _ => panic!("wrong op type"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Addi {
    rd: usize,
    rs1: usize,
    imm: i32,
}

impl Arbitrary for Addi {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..32usize, any::<Signed<12, 0>>())
            .prop_map(|(rd, rs1, imm)| Addi {
                rd,
                rs1,
                imm: imm.into(),
            })
            .boxed()
    }
}

impl Op for Addi {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let imm = (self.imm as u32) << 20;
        let rs1 = (self.rs1 << 15) as u32;
        let rd = (self.rd << 7) as u32;
        0b0010011 | imm | rs1 | rd
    }
}

impl From<rvsim::Op> for Addi {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Addi { rd, rs1, i_imm } => Self {
                rd,
                rs1,
                imm: i_imm,
            },
            _ => panic!("wrong op type"),
        }
    }
}

// pub type Jal = Lui;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Jal {
    pub rd: usize,
    pub offset: i32,
}

impl Arbitrary for Jal {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, any::<Signed<19, 2>>())
            .prop_map(|(rd, offset)| {
                let mut offset = offset.into();
                if offset == 0 {
                    offset = 4;
                }
                Jal { rd, offset }
            })
            .boxed()
    }
}

impl Op for Jal {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let b12_19 = (self.offset as u32) & 0xff000;
        let b_10_1 = ((self.offset as u32) & 0x7fe) << 20;
        let b_11 = (self.offset as u32 & (1 << 11)) << 9;
        let b_20 = (self.offset as u32 & (1 << 20)) << 11;
        let rd = (self.rd << 7) as u32;
        0b1101111 | b12_19 | b_10_1 | b_11 | b_20 | rd
    }
}

impl From<rvsim::Op> for Jal {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Jal { rd, j_imm } => Self { rd, offset: j_imm },
            _ => panic!("wrong op type"),
        }
    }
}

// pub type Jalr = Jal;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Jalr {
    pub rd: usize,
    pub rs1: usize,
    pub imm: i32,
}

impl Arbitrary for Jalr {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (0..32usize, 0..32usize, any::<Signed<12, 0>>())
            .prop_map(|(rd, rs1, offset)| {
                let imm = i32::from(offset);
                let imm = imm - imm % 4;
                assert_eq!(imm % 4, 0);
                Jalr { rd, rs1, imm }
            })
            .boxed()
    }
}

impl Op for Jalr {
    fn execute<E>(&self, mut state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        let mut rs1 = state.regs[self.rs1] as i32;
        // FIX: the result should be 4-bytes aligned
        // this is a dirty hack to make it work without ad-hoc strategies
        // but does not cover all possible cases
        rs1 = rs1 - rs1 % 4;
        state.regs[self.rs1] = rs1 as u32;
        assert_eq!(state.regs[self.rs1] % 4, 0);
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let imm = (self.imm as u32) << 20;
        let rs1 = (self.rs1 << 15) as u32;
        let rd = (self.rd << 7) as u32;
        0b1100111 | imm | rs1 | rd
    }
}

impl From<rvsim::Op> for Jalr {
    fn from(other: rvsim::Op) -> Self {
        match other {
            rvsim::Op::Jalr { rd, rs1, i_imm } => Self {
                rd,
                rs1,
                imm: i_imm,
            },
            _ => panic!("wrong op type"),
        }
    }
}

#[derive(Debug, Clone)]
struct Signed<const N: usize, const OFFSET: usize> {
    inner: i32,
}

impl<const N: usize, const OFFSET: usize> Arbitrary for Signed<N, OFFSET> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (-1i32 << (N - 1)..(1i32 << (N - 1)))
            .prop_map(|inner| Self {
                inner: inner << OFFSET,
            })
            .boxed()
    }
}

impl<const N: usize, const OFFSET: usize> From<Signed<N, OFFSET>> for i32 {
    fn from(other: Signed<N, OFFSET>) -> Self {
        other.inner as i32
    }
}
