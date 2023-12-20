use super::*;
use quickcheck::{Arbitrary as _, Gen};

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
    ($op:expr, $state:expr, $pc:expr) => {{
        let mut mem = load_op_at_addr($pc, $op);
        let mut cpu_state = rvsim::CpuState::new($pc);
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
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let uimm = Uimm::arbitrary(g).inner;
        Self { rd, uimm }
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

impl From<Lui> for rvsim::Op {
    fn from(other: Lui) -> rvsim::Op {
        rvsim::Op::Lui {
            rd: other.rd,
            u_imm: other.uimm,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Auipc {
    rd: usize,
    uimm: i32,
    // FIX: is pc signed?
    pc: u32,
}

impl Arbitrary for Auipc {
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let uimm = Uimm::arbitrary(g).inner;
        let pc = u32::arbitrary(g);
        Self { rd, uimm, pc }
    }
}

impl Op for Auipc {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state, self.pc)
    }

    fn to_op(&self) -> u32 {
        let uimm = (self.uimm as u32) & 0xfffff000;
        let rd = (self.rd << 7) as u32;
        0b0010111 | uimm | rd
    }
}

impl From<Auipc> for rvsim::Op {
    fn from(other: Auipc) -> rvsim::Op {
        rvsim::Op::Auipc {
            rd: other.rd,
            u_imm: other.uimm,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Addi {
    rd: usize,
    rs1: usize,
    rs1_val: i32,
    imm: i32,
}

impl Arbitrary for Addi {
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let rs1 = Reg::arbitrary(g).inner;
        let imm = Iimm::arbitrary(g).inner;
        let rs1_val = i32::arbitrary(g);
        Self {
            rd,
            rs1,
            imm,
            rs1_val,
        }
    }
}

impl Op for Addi {
    fn execute<E>(&self, mut state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        state.regs[self.rs1] = self.rs1_val as u32;
        execute!(self, state)
    }

    fn to_op(&self) -> u32 {
        let imm = (self.imm as u32) << 20;
        let rs1 = (self.rs1 << 15) as u32;
        let rd = (self.rd << 7) as u32;
        0b0010011 | imm | rs1 | rd
    }
}

impl From<Addi> for rvsim::Op {
    fn from(other: Addi) -> rvsim::Op {
        rvsim::Op::Addi {
            rd: other.rd,
            rs1: other.rs1,
            i_imm: other.imm,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Jal {
    rd: usize,
    offset: i32,
    pc: u32,
}

impl Arbitrary for Jal {
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let offset = JalOffset::arbitrary(g).inner;
        let pc = u32::arbitrary(g);
        Self { rd, offset, pc }
    }
}

impl Op for Jal {
    fn execute<E>(&self, state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        execute!(self, state, self.pc)
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

impl From<Jal> for rvsim::Op {
    fn from(other: Jal) -> rvsim::Op {
        rvsim::Op::Jal {
            rd: other.rd,
            j_imm: other.offset,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Jalr {
    rd: usize,
    rs1: usize,
    rs1_value: i32,
    imm: i32,
    pc: u32,
}

impl Arbitrary for Jalr {
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let rs1 = Reg::arbitrary(g).inner;
        let imm = Iimm::arbitrary(g).inner;
        let mut rs1_value = i32::arbitrary(g);
        let res = imm.wrapping_add(rs1_value);
        if res & 2 != 0 {
            rs1_value = rs1_value.wrapping_sub(2);
        }
        let pc = u32::arbitrary(g);
        Self {
            rd,
            rs1,
            rs1_value,
            imm,
            pc,
        }
    }
}

impl Op for Jalr {
    fn execute<E>(&self, mut state: CpuState) -> TraceTable<E>
    where
        E: StarkField,
    {
        state.regs[self.rs1] = self.rs1_value as u32;
        execute!(self, state, self.pc)
    }

    fn to_op(&self) -> u32 {
        let imm = (self.imm as u32) << 20;
        let rs1 = (self.rs1 << 15) as u32;
        let rd = (self.rd << 7) as u32;
        0b1100111 | imm | rs1 | rd
    }
}

impl From<Jalr> for rvsim::Op {
    fn from(other: Jalr) -> rvsim::Op {
        rvsim::Op::Jalr {
            rd: other.rd,
            rs1: other.rs1,
            i_imm: other.imm,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Slti {
    rd: usize,
    rs1: usize,
    rs1_value: i32,
    imm: i32,
}

impl Arbitrary for Slti {
    fn arbitrary(g: &mut Gen) -> Self {
        let rd = Reg::arbitrary(g).inner;
        let rs1 = Reg::arbitrary(g).inner;
        let imm = Iimm::arbitrary(g).inner;
        let rs1_value = i32::arbitrary(g);
        Self {
            rd,
            rs1,
            imm,
            rs1_value,
        }
    }
}

impl Op for Slti {
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
        0b0010011 | imm | rs1 | rd | 0b010 << 12
    }
}

impl From<Slti> for rvsim::Op {
    fn from(other: Slti) -> rvsim::Op {
        rvsim::Op::Slti {
            rd: other.rd,
            rs1: other.rs1,
            i_imm: other.imm,
        }
    }
}

// Signed U immediate
#[derive(Debug, Clone)]
struct Uimm {
    inner: i32,
}

impl quickcheck::Arbitrary for Uimm {
    fn arbitrary(g: &mut Gen) -> Self {
        let inner = i32::arbitrary(g) & 0xfffff000u32 as i32;
        Self { inner }
    }
}

#[derive(Debug, Clone)]
struct Iimm {
    inner: i32,
}

impl quickcheck::Arbitrary for Iimm {
    fn arbitrary(g: &mut Gen) -> Self {
        const BOUNDARIES: &[i32] = &[-2048, 0, 2047];
        let inner = if biased(g, 10) {
            *g.choose(BOUNDARIES).unwrap()
        } else {
            i32::arbitrary(g) & 0xfff
        };

        Self { inner }
    }
}

#[derive(Debug, Clone)]
struct Reg {
    inner: usize,
}

impl quickcheck::Arbitrary for Reg {
    fn arbitrary(g: &mut Gen) -> Self {
        let regs = (0..32).collect::<Vec<_>>();
        let inner = *g.choose(&regs).unwrap();
        Self { inner }
    }
}

#[derive(Debug, Clone)]
struct JalOffset {
    inner: i32,
}

impl quickcheck::Arbitrary for JalOffset {
    fn arbitrary(g: &mut Gen) -> Self {
        const BOUNDARIES: &[i32] = &[-524288, 0, 524287];
        let mut inner = if biased(g, 10) {
            *g.choose(BOUNDARIES).unwrap()
        } else {
            i32::arbitrary(g) & 0xfffff
        };
        inner = inner - inner % 2;
        Self { inner }
    }
}

// biased coin
// prob = 0..=100
fn biased(g: &mut Gen, prob: u8) -> bool {
    assert!(prob <= 100);
    let prob = prob as u64 * 255 / 100;
    assert!(prob <= 255);
    u8::arbitrary(g) <= prob as u8
}
