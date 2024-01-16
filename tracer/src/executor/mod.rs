mod cpu;
pub mod memory;
mod program;

pub use program::Program;
use std::time::Instant;

use crate::{
    executor::{cpu::Cpu, memory::Memory},
    trace::TraceTable,
    Felem,
};

// for some reason the trace length must be at least 8
const MIN_LEN: usize = 8;

pub fn exec(program: &Program) -> TraceTable<Felem> {
    let mut memory = Memory::new();
    let start = Instant::now();
    let cpu = Cpu::run(program, &mut memory);
    log::debug!(
        "program loaded and executed in {} cycles / {} ms",
        cpu.trace_len(),
        start.elapsed().as_millis()
    );
    let cpu_trace_len = cpu.trace_len();
    let memory_trace_len = memory.trace_len();
    let trace_len = core::cmp::max(cpu_trace_len, memory_trace_len).next_power_of_two();
    let trace_len = core::cmp::max(trace_len, MIN_LEN);

    assert!(
        trace_len > 1,
        "the trace length was {trace_len}, maybe something went wrong?",
    );

    let cpu_trace = cpu.into_trace(trace_len);
    let (mem_trace, aux_trace) = memory.into_trace(trace_len);
    let trace = cpu_trace.into_iter().chain(mem_trace).collect::<Vec<_>>();

    TraceTable::new(trace, aux_trace)
}
