use crate::{
    executor::{Memory, Program},
    Felem,
};
use rand::Rng;
use rvsim::{Clock, CpuState, Interp, Op, SimpleClock};
use trace_defs::{
    BODY, CPU_TRACE_WIDTH, CYCLE, FUNCT7_ZERO, H_0, H_1, H_2, H_3, INSN, INS_END, LOADING, PC,
    RD_BITS_END, RD_ZERO, RS1_BITS_END, RS2_BITS_END,
};
use winterfell::math::FieldElement;

const ONE: Felem = Felem::ONE;
const ZERO: Felem = Felem::ZERO;
const MIN_RAND_ROWS: usize = 1;

pub struct Cpu {
    state: CpuState,
    clock: SimpleClock,
    trace: [Vec<Felem>; CPU_TRACE_WIDTH],
}

impl Cpu {
    pub fn current_trace(&self, memory: &Memory) -> [Felem; CPU_TRACE_WIDTH] {
        let mut trace = [0u32.into(); CPU_TRACE_WIDTH];
        trace[PC] = signed(self.state.pc);
        let insn = self.insn_at_pc(memory);
        Self::save_u32_to_bits(&mut trace[INS_END..], insn);
        trace[INSN] = insn.into();
        trace[BODY] = ONE;
        let clock = self.clock.read_cycle();
        trace[CYCLE] = clock.into();
        trace
    }

    pub fn run(program: &Program, memory: &mut Memory) -> Self {
        let mut cpu = Self::load_program(program, memory);
        cpu.run_inner(memory);
        cpu
    }

    fn run_inner(&mut self, memory: &mut Memory) {
        let mut current_trace = self.current_trace(memory);
        loop {
            // load source registers from memory
            let rs1 = self.next_rs1(memory);
            let rs1 = memory.register_file().load(rs1);
            let rs2 = self.next_rs2(memory);
            let rs2 = memory.register_file().load(rs2);

            let rd_idx = self.next_rd(memory) as usize;
            // save register contentes to trace
            Self::save_u32_to_bits(&mut current_trace[RS1_BITS_END..], rs1);
            Self::save_u32_to_bits(&mut current_trace[RS2_BITS_END..], rs2);
            if rd_idx == 0 {
                current_trace[RD_ZERO] = ONE;
            }
            if current_trace[INS_END..INS_END + 7]
                .iter()
                .all(|&b| b == ZERO)
            {
                current_trace[FUNCT7_ZERO] = ONE;
            }

            // Add current row to trace
            for (col, val) in self.trace.iter_mut().zip(current_trace) {
                col.push(val);
            }

            let prev = self.state.clone();
            match self.interp(memory).step() {
                Ok(op) => {
                    log::trace!("executed {:?} | pc: {:X}", op, self.state.pc);
                    current_trace = self.current_trace(memory);

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
                        Op::Add { rs1, rs2, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            let rs2 = prev.x[rs2] as i32;
                            // TODO: this is essentially re-doing an addition
                            current_trace[H_0] = signed_overflow(rs1, rs2);
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
                            current_trace[H_2] = ((i_imm as u32 ^ rs1 as u32) & 1).into();
                            let rs1 = rs1 as u32;
                            for i in 0..32 {
                                current_trace[RS1_BITS_END + i] = ((rs1 >> (31 - i)) & 1).into();
                            }
                        }
                        Op::Bne {
                            rs1, rs2, b_imm, ..
                        } => {
                            let pc = prev.pc as i32;
                            let rs1 = prev.x[rs1];
                            let rs2 = prev.x[rs2];
                            if rs1 != rs2 {
                                current_trace[H_0] =
                                    ONE / (signed::<Felem>(rs1) - signed::<Felem>(rs2));
                                current_trace[H_1] = signed_overflow(pc, b_imm);
                            } else {
                                current_trace[H_1] = signed_overflow(pc, 4);
                            }
                        }
                        Op::Slti { i_imm, rs1, .. } => {
                            let rs1 = prev.x[rs1] as i32;
                            // FIXME: this is a workaround for the fact that we don't have constraints
                            // on h0 > 0, which would make the first constraint valid for rd = 1 when rs1 = i_imm
                            current_trace[H_0] = ONE;
                            if rs1 < i_imm {
                                current_trace[H_0] = ((i_imm as i64 - rs1 as i64) as u32).into();
                            } else {
                                current_trace[H_1] = ((rs1 as i64 - i_imm as i64) as u32).into();
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

            // save previous destination register to memory
            let rd = self.state.x[rd_idx];
            memory.register_file().store(rd_idx as u32, rd);
            memory.advance_bus_clk();
            Self::save_u32_to_bits(&mut current_trace[RD_BITS_END..], rd);
        }
        log::trace!("completed in {} cycles", self.clock.read_cycle());
    }

    fn interp<'m>(&mut self, memory: &'m mut Memory) -> Interp<'_, 'm, '_, Memory, SimpleClock> {
        Interp::new(&mut self.state, memory, &mut self.clock)
    }

    fn insn_at_pc(&self, memory: &Memory) -> u32 {
        memory.get(self.state.pc)
    }

    // rs1 used by next instruction
    fn next_rs1(&self, memory: &Memory) -> u32 {
        self.insn_at_pc(memory) >> 15 & 0x1f
    }

    // rs2 used by next instruction
    fn next_rs2(&self, memory: &Memory) -> u32 {
        self.insn_at_pc(memory) >> 20 & 0x1f
    }

    // rd used by next instruction
    fn next_rd(&self, memory: &Memory) -> u32 {
        self.insn_at_pc(memory) >> 7 & 0x1f
    }

    #[allow(clippy::needless_range_loop)]
    fn save_u32_to_bits<E: From<u32>>(trace: &mut [E], val: u32) {
        assert!(trace.len() >= 32);
        for i in 0..32 {
            trace[i] = ((val >> (31 - i)) & 1).into();
        }
    }

    // This is not strictly part of the role of the CPU, but it's convenient to have it here since it
    // reuses the same columns of the CPU trace.
    // We can probably substitute this by proving we execute a simple elf loader.
    pub fn load_program(program: &Program, memory: &mut Memory) -> Self {
        let mut trace: [Vec<Felem>; CPU_TRACE_WIDTH] = core::array::from_fn(|_| Vec::new());

        let words = program.segments().iter().flat_map(|(addr, segment)| {
            assert!(segment.len() % 4 == 0);
            segment.chunks_exact(4).enumerate().map(|(i, bytes)| {
                (
                    *addr + i as u32 * 4,
                    u32::from_le_bytes(bytes.try_into().unwrap()),
                )
            })
        });

        // TODO: we can make this more efficient as bytes are likely contiguous

        for (addr, word) in words {
            memory.store(addr, word);
            let mut row = [ZERO; CPU_TRACE_WIDTH];
            row[PC] = addr.into();
            row[INSN] = word.into();
            row[CYCLE] = memory.bus_clock().into();
            row[LOADING] = ONE;
            for (col, val) in trace.iter_mut().zip(row) {
                col.push(val)
            }

            memory.advance_bus_clk();
        }

        let state = CpuState::new(program.entrypoint());
        let mut clock = SimpleClock::new();
        clock.instret = memory.bus_clock() as u64;
        log::trace!("loading completed in {} cycles", clock.instret);
        Self {
            state,
            clock,
            trace,
        }
    }

    pub fn trace_len(&self) -> usize {
        self.trace[0].len() + MIN_RAND_ROWS
    }

    fn pad_column(col: &mut Vec<Felem>, index: usize, pad: usize) {
        match index {
            BODY | LOADING | H_3 => {
                col.extend(vec![Felem::ZERO; pad]);
            }
            CYCLE => {
                let start = *col.last().unwrap();
                col.extend(
                    core::iter::successors(Some(start + ONE), |prev| Some(*prev + ONE)).take(pad),
                );
            }
            _ => {
                let mut bytes = vec![0u32; pad];
                rand::thread_rng().fill(&mut bytes[..]);
                col.extend(bytes.iter().map(|&b| Felem::from(b)));
            }
        }
    }

    fn into_trace_inner(self, trace_len: usize) -> [Vec<Felem>; CPU_TRACE_WIDTH] {
        let mut trace = self.trace;

        for (i, column) in trace.iter_mut().enumerate() {
            let pad = trace_len - column.len();
            Self::pad_column(column, i, pad);
        }

        trace
    }

    pub fn into_trace(self, trace_len: usize) -> [Vec<Felem>; CPU_TRACE_WIDTH] {
        assert!(trace_len.is_power_of_two());
        assert!(trace_len >= self.trace_len());

        self.into_trace_inner(trace_len)
    }

    pub fn into_trace_with_splits(
        self,
        n_segments: usize,
        segment_len: usize,
    ) -> (
        Vec<[Vec<Felem>; CPU_TRACE_WIDTH]>,
        [Vec<Felem>; CPU_TRACE_WIDTH],
    ) {
        assert!(segment_len.is_power_of_two());

        // the first segment can hold segment_len - 1 rows from the original execution (the last one in padding)
        // while successive segments can only hold segment_len - 2 rows from the original execution (1 is used for padding and 1 is the last row of the previous segment)
        let trace_len = n_segments * (segment_len - 2) + 1;
        assert!(trace_len >= self.trace_len());
        let trace_len = trace_len.next_power_of_two();

        let full_trace = self.into_trace_inner(trace_len);

        (
            super::utils::split_trace_with_padding::<CPU_TRACE_WIDTH, _>(
                &full_trace,
                n_segments,
                segment_len,
            ),
            full_trace,
        )
    }
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
