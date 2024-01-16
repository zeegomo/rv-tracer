use crate::trace::TraceTable;
use memory::Memory;
use rand::Rng;
use rvsim::elf::Elf32;
use rvsim::*;
use trace_defs::{
    BODY, CHIPLETS_START, CYCLE, H_0, H_1, H_2, INS_END, LOADING, MAIN_TRACE_WIDTH, PC,
    PC_CONTENTS, RD_BITS_END, READING_PC, RS1_BITS_END, RS2_BITS_END, UNSIGNED_PC,
};
use winterfell::math::{fields::f64::BaseElement, FieldElement};

pub mod memory;

// for some reason the trace length must be at least 8
const MIN_LEN: usize = 8;

pub struct Tracer {
    memory: Memory,
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
        let memory = Memory::new();
        Self {
            memory,
            state,
            executed: Vec::new(),
            clock,
            data,
        }
    }

    #[allow(clippy::needless_range_loop)]
    pub fn current_trace<E>(&mut self) -> [E; MAIN_TRACE_WIDTH]
    where
        E: FieldElement,
    {
        let mut trace = [0u32.into(); MAIN_TRACE_WIDTH];
        for i in 0..32 {
            trace[i] = signed(self.state.x[i]);
        }
        trace[PC] = signed(self.state.pc);
        trace[UNSIGNED_PC] = self.state.pc.into();
        trace[READING_PC] = E::ONE;
        let pc = self.insn_at_pc();
        Self::save_u32_to_bits(&mut trace[INS_END..], pc);
        trace[PC_CONTENTS] = pc.into();
        let clock = self.clock.read_cycle();
        trace[BODY] = E::ONE;
        trace[CYCLE] = clock.into();
        assert!(clock < 100);
        trace
    }
    pub fn run<E>(&mut self) -> Vec<Vec<E>>
    where
        E: FieldElement,
    {
        let mut trace = vec![Vec::new(); MAIN_TRACE_WIDTH];
        let mut rd_idx = 0;
        let mut current_trace = self.current_trace();
        loop {
            // Save current state to trace
            let rs1 = self.state.x[self.next_rs1() as usize];
            let rs2 = self.state.x[self.next_rs2() as usize];
            let rd = self.state.x[rd_idx];
            rd_idx = self.next_rd() as usize;
            Self::save_u32_to_bits(&mut current_trace[RS1_BITS_END..], rs1);
            Self::save_u32_to_bits(&mut current_trace[RS2_BITS_END..], rs2);
            Self::save_u32_to_bits(&mut current_trace[RD_BITS_END..], rd);

            for i in 0..MAIN_TRACE_WIDTH {
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
                            current_trace[H_0] = signed_overflow(pc, u_imm);
                        }
                        Op::Addi { i_imm, rs1, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            // TODO: this is essentially re-doing an addition
                            current_trace[H_0] = signed_overflow(rs1, i_imm);
                        }
                        Op::Jal { j_imm, .. } => {
                            let pc = prev.pc as i32;
                            current_trace[H_0] = signed_overflow(pc, 4);
                            current_trace[H_1] = signed_overflow(j_imm, pc);
                        }
                        Op::Jalr { i_imm, rs1, .. } => {
                            let pc = prev.pc as i32;
                            let rs1 = prev.x[rs1] as i32;
                            current_trace[H_0] = signed_overflow(pc, 4);
                            current_trace[H_1] = signed_overflow(rs1, i_imm);
                            current_trace[H_2] = E::from((i_imm as u32 ^ rs1 as u32) & 1);
                            let rs1 = rs1 as u32;
                            for i in 0..32 {
                                current_trace[RS1_BITS_END + i] = ((rs1 >> (31 - i)) & 1).into();
                            }
                        }
                        Op::Slti { i_imm, rs1, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            // FIXME: this is a workaround for the fact that we don't have constraints
                            // on h0 > 0, which would make the first constraint valid for rd = 1 when rs1 = i_imm
                            current_trace[H_0] = E::ONE;
                            if rs1 < i_imm {
                                current_trace[H_0] = E::from((i_imm as i64 - rs1 as i64) as u32);
                            } else {
                                current_trace[H_1] = E::from((rs1 as i64 - i_imm as i64) as u32);
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
            self.memory.advance();
        }
        trace
    }

    fn interp(&mut self) -> Interp<'_, '_, '_, Memory, SimpleClock> {
        Interp::new(&mut self.state, &mut self.memory, &mut self.clock)
    }

    fn insn_at_pc(&self) -> u32 {
        self.memory.get(self.state.pc)
    }

    // rs1 used by next instruction
    fn next_rs1(&self) -> u32 {
        self.insn_at_pc() >> 15 & 0x1f
    }

    // rs2 used by next instruction
    fn next_rs2(&self) -> u32 {
        self.insn_at_pc() >> 20 & 0x1f
    }

    // rd used by next instruction
    fn next_rd(&self) -> u32 {
        self.insn_at_pc() >> 7 & 0x1f
    }

    fn save_u32_to_bits<E: FieldElement>(trace: &mut [E], val: u32) {
        assert!(trace.len() >= 32);
        for i in 0..32 {
            trace[i] = ((val >> (31 - i)) & 1).into();
        }
    }

    /// Builds an execution trace for computing a Fibonacci sequence of the specified length such
    /// that each row advances the sequence by 2 terms.
    pub fn build_trace(mut self) -> TraceTable<BaseElement> {
        let mut trace = self.load_program_to_memory();
        log::debug!("loading completed in {} cycles", self.clock.read_cycle());
        let stack_trace = self.run::<BaseElement>();
        for (trace, stack_trace) in trace.iter_mut().zip(stack_trace) {
            trace.extend(stack_trace);
        }
        let trace_len = trace[0].len();
        log::debug!("program completed in {} cycles", trace_len);
        assert!(
            trace_len > 1,
            "the trace length was {trace_len}, maybe something went wrong?",
        );
        let memory_trace_len = self.memory.trace_len();
        log::trace!(
            "trace length stack/memory: {}/{}",
            trace_len,
            memory_trace_len
        );
        let trace_len = core::cmp::max(trace_len, memory_trace_len);

        let next_power_of_two = core::cmp::max(trace_len.next_power_of_two(), MIN_LEN);
        // TODO: we neeed at least 1 row of padding
        log::debug!("padding trace to {} cycles", next_power_of_two);
        // TODO: proper padding
        for (i, column) in trace.iter_mut().enumerate() {
            let pad = next_power_of_two - column.len();
            if i == BODY || i == LOADING || i == READING_PC {
                if i != READING_PC {
                    *column.last_mut().unwrap() = BaseElement::ZERO;
                }
                column.extend(vec![BaseElement::ZERO; pad]);
            } else {
                let mut bytes = vec![0u32; pad];
                rand::thread_rng().fill(&mut bytes[..]);
                column.extend(bytes.iter().map(|&b| BaseElement::from(b)));
            }
        }
        let mem_trace = self
            .memory
            .to_trace(next_power_of_two, next_power_of_two - memory_trace_len);

        for (trace, mem_trace) in trace.iter_mut().skip(CHIPLETS_START).zip(mem_trace.0) {
            *trace = mem_trace;
        }

        TraceTable::new(trace, mem_trace.1)
    }

    fn load_program_to_memory(&mut self) -> Vec<Vec<BaseElement>> {
        let mut trace = vec![vec![]; MAIN_TRACE_WIDTH];

        let bytes = self.data.data.iter().flat_map(|(addr, segment)| {
            assert!(segment.len() % 4 == 0);
            segment.chunks_exact(4).enumerate().map(|(i, bytes)| {
                (
                    *addr + i as u32 * 4,
                    u32::from_le_bytes(bytes.try_into().unwrap()),
                )
            })
        });

        // TODO: we can make this more efficient as bytes are likely contiguous
        for (addr, byte) in bytes {
            self.memory.store(addr, byte);
            let mut row = vec![BaseElement::ZERO; MAIN_TRACE_WIDTH];
            row[PC] = addr.into();
            row[PC_CONTENTS] = byte.into();
            row[CYCLE] = self.memory.clock().into();
            row[READING_PC] = BaseElement::ONE;
            row[LOADING] = BaseElement::ONE;
            for (col, val) in trace.iter_mut().zip(row) {
                col.push(val)
            }
            self.memory.advance();
        }

        self.clock.instret = self.memory.clock() as u64;
        trace
    }
}

pub fn sim(elf: Elf32) -> TraceTable<BaseElement> {
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
