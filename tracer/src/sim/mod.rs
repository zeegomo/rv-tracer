use memory::SimpleMemory;
use rand::Rng;
use rvsim::elf::Elf32;
use rvsim::*;
use trace::RS1_BITS_END;
use trace_defs::{self as trace, TRACE_WIDTH};
use winterfell::{
    math::{FieldElement, StarkField},
    TraceTable,
};

pub mod memory;

// for some reason the trace length must be at least 8
const MIN_LEN: usize = 8;

pub struct Tracer {
    memory: SimpleMemory,
    state: CpuState,
    executed: Vec<Op>,
    clock: SimpleClock,
    data: LoadData,
}

pub struct LoadData {
    // a pair of (addr, segment)
    data: Vec<(u32, Vec<u8>)>,
}

impl LoadData {
    pub fn new(data: Vec<(u32, Vec<u8>)>) -> Self {
        Self { data }
    }
}

impl<'a> From<Elf32<'a>> for LoadData {
    fn from(elf: Elf32<'a>) -> Self {
        if elf.ident.data != elf::ELF_IDENT_DATA_2LSB
            || elf.ident.abi != elf::ELF_IDENT_ABI_SYSV
            || elf.header.typ != elf::ELF_TYPE_EXECUTABLE
            || elf.header.machine != elf::ELF_MACHINE_RISCV
        {
            panic!("unsupported executable format");
        }

        let mut data = LoadData { data: Vec::new() };
        for (i, ph) in elf.ph.iter().enumerate() {
            let addr = ph.vaddr;
            if ph.typ == rvsim::elf::ELF_PROGRAM_TYPE_LOADABLE {
                data.data.push((addr, elf.p[i].to_vec()));
            }
        }
        data
    }
}

impl Tracer {
    pub fn new(state: CpuState, data: LoadData) -> Self {
        let clock = rvsim::SimpleClock::new();
        let memory = SimpleMemory::new();
        Self {
            memory,
            state,
            executed: Vec::new(),
            clock,
            data,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn current_trace<E>(&mut self) -> [E; TRACE_WIDTH]
    where
        E: FieldElement,
    {
        let mut trace = [0u32.into(); TRACE_WIDTH];
        for i in 0..32 {
            trace[i] = signed(self.state.x[i]);
        }
        trace[trace::PC] = signed(self.state.pc);
        let pc = self.insn_at_pc();
        Self::save_u32_to_bits(&mut trace[trace::INS_END..], pc);
        let clock = self.clock.read_cycle();
        trace[trace::BODY] = E::ONE;
        trace[trace::CYCLE] = clock.into();
        assert!(clock < 100);
        trace
    }
    pub fn run<E>(&mut self) -> Vec<Vec<E>>
    where
        E: FieldElement,
    {
        let mut trace = vec![Vec::new(); TRACE_WIDTH];
        let mut rd_idx = 0;
        let mut current_trace = self.current_trace();
        loop {
            // Save current state to trace
            let rs1 = self.state.x[self.next_rs1() as usize];
            let rs2 = self.state.x[self.next_rs2() as usize];
            let rd = self.state.x[rd_idx];
            rd_idx = self.next_rd() as usize;
            Self::save_u32_to_bits(&mut current_trace[trace::RS1_BITS_END..], rs1);
            Self::save_u32_to_bits(&mut current_trace[trace::RS2_BITS_END..], rs2);
            Self::save_u32_to_bits(&mut current_trace[trace::RD_BITS_END..], rd);

            for i in 0..TRACE_WIDTH {
                trace[i].push(current_trace[i]);
            }
            let prev = self.state.clone();
            match self.interp().step() {
                Ok(op) => {
                    self.executed.push(op);
                    log::trace!("executed {:?}", op);
                    current_trace = self.current_trace();

                    match op {
                        Op::Auipc { u_imm, .. } => {
                            let pc = prev.pc as i32;
                            // TODO: this is essentially re-doing an addition
                            current_trace[trace::H_0] = signed_overflow(pc, u_imm);
                        }
                        Op::Addi { i_imm, rs1, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            // TODO: this is essentially re-doing an addition
                            current_trace[trace::H_0] = signed_overflow(rs1, i_imm);
                        }
                        Op::Jal { j_imm, .. } => {
                            let pc = prev.pc as i32;
                            current_trace[trace::H_0] = signed_overflow(pc, 4);
                            current_trace[trace::H_1] = signed_overflow(j_imm, pc);
                        }
                        Op::Jalr { i_imm, rs1, .. } => {
                            let pc = prev.pc as i32;
                            let rs1 = prev.x[rs1] as i32;
                            current_trace[trace::H_0] = signed_overflow(pc, 4);
                            current_trace[trace::H_1] = signed_overflow(rs1, i_imm);
                            current_trace[trace::H_2] = E::from((i_imm as u32 ^ rs1 as u32) & 1);
                            let rs1 = rs1 as u32;
                            for i in 0..32 {
                                current_trace[RS1_BITS_END + i] = ((rs1 >> (31 - i)) & 1).into();
                            }
                        }
                        Op::Slti { i_imm, rs1, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            // FIXME: this is a workaround for the fact that we don't have constraints
                            // on h0 > 0, which would make the first constraint valid for rd = 1 when rs1 = i_imm
                            current_trace[trace::H_0] = E::ONE;
                            if rs1 < i_imm {
                                current_trace[trace::H_0] =
                                    E::from((i_imm as i64 - rs1 as i64) as u32);
                            } else {
                                current_trace[trace::H_1] =
                                    E::from((rs1 as i64 - i_imm as i64) as u32);
                            }
                        }
                        _ => {}
                    }
                }
                Err((rvsim::CpuError::IllegalInstruction, _)) => {
                    break;
                }
                Err(e) => {
                    log::error!("execution halted due to: {:?}", e);
                    break;
                }
            }
        }
        trace
    }

    fn interp(&mut self) -> Interp<'_, '_, '_, SimpleMemory, SimpleClock> {
        Interp::new(&mut self.state, &mut self.memory, &mut self.clock)
    }

    fn insn_at_pc(&mut self) -> u32 {
        let pc = self.state.pc;
        let mut insn = 0u32;
        self.interp().mem.access(pc, MemoryAccess::Load(&mut insn));
        insn
    }

    // rs1 used by next instruction
    fn next_rs1(&mut self) -> u32 {
        self.insn_at_pc() >> 15 & 0x1f
    }

    // rs2 used by next instruction
    fn next_rs2(&mut self) -> u32 {
        self.insn_at_pc() >> 20 & 0x1f
    }

    // rd used by next instruction
    fn next_rd(&mut self) -> u32 {
        self.insn_at_pc() >> 7 & 0x1f
    }

    #[allow(clippy::needless_range_loop)]
    fn save_u32_to_bits<E: FieldElement>(trace: &mut [E], val: u32) {
        assert!(trace.len() >= 32);
        for i in 0..32 {
            trace[i] = ((val >> (31 - i)) & 1).into();
        }
    }

    /// Builds an execution trace for computing a Fibonacci sequence of the specified length such
    /// that each row advances the sequence by 2 terms.
    pub fn build_trace<E: StarkField>(mut self) -> TraceTable<E> {
        let mut trace = Self::load_program_to_memory(&self.data, &mut self.memory);
        let stack_trace = self.run::<E>();
        for (trace, stack_trace) in trace.iter_mut().zip(stack_trace) {
            trace.extend(stack_trace);
        }
        let trace_len = trace[0].len();
        log::debug!("program completed in {} cycles", trace_len);
        assert!(
            trace_len > 1,
            "the trace length was {trace_len}, maybe something went wrong?",
        );
        let next_power_of_two = core::cmp::max(trace_len.next_power_of_two(), MIN_LEN);
        // TODO: we neeed at least 1 row of padding
        log::debug!("padding trace to {} cycles", next_power_of_two);
        // TODO: proper padding
        for (i, column) in trace.iter_mut().enumerate() {
            let pad = next_power_of_two - column.len();
            if i == trace::BODY {
                *column.last_mut().unwrap() = E::ZERO;
                column.extend(vec![E::ZERO; pad]);
            } else {
                let mut bytes = vec![0u32; pad];
                rand::thread_rng().fill(&mut bytes[..]);
                column.extend(bytes.iter().map(|&b| E::from(b)));
            }
        }
        TraceTable::init(trace)
    }

    fn load_program_to_memory<E: Clone>(data: &LoadData, memory: &mut SimpleMemory) -> Vec<Vec<E>> {
        for (addr, segment) in &data.data {
            memory.load_slice(*addr, segment);
        }
        // TODO: this should generate some trace to verify it was loaded properly, but it mosly depends on memory
        // constraints which will be added later
        let trace = vec![vec![]; TRACE_WIDTH];
        trace
    }
}

pub fn sim<E: StarkField>(elf: Elf32) -> TraceTable<E> {
    // Create the virtual CPU state, setting the PC to the start of our program.
    let state = rvsim::CpuState::new(elf.header.entry);

    let tracer = Tracer::new(state, elf.into());
    tracer.build_trace()
}

fn signed<E: FieldElement>(val: u32) -> E {
    if val & (1u32 << 31) == 0 {
        E::from(val)
    } else {
        E::from(val) - E::from(2u32) * E::from(1u32 << 31)
    }
}

fn signed_overflow<E: FieldElement>(a: i32, b: i32) -> E {
    let (res, overflow) = a.overflowing_add(b);

    match (overflow, res >= 0) {
        (true, true) => -E::from(1u32),
        (true, false) => E::from(1u32),
        _ => E::ZERO,
    }
}
